// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{ActionHandler, ActionRequest, Rect, TreeUpdate};
use accesskit_consumer::Tree;
use async_executor::{LocalExecutor, Task};
use async_io::Async;
use async_net::unix::UnixStream as AsyncUnixStream;
use futures_channel::{mpsc, oneshot};
use futures_lite::{future::block_on, pin, prelude::*};
use futures_util::{
    future::{join_all, select, select_all, Either},
    sink::SinkExt,
};
use rustix::net::{recvmsg, RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags};
use std::{
    cell::RefCell,
    io::{self, IoSliceMut},
    os::{
        fd::{AsFd, BorrowedFd, OwnedFd},
        unix::net::UnixDatagram,
    },
    rc::Rc,
    thread::JoinHandle,
};

async fn tree_stream_task(
    mut stream: AsyncUnixStream,
    tree: &RefCell<Tree>,
    tree_update_txs: Rc<RefCell<Vec<mpsc::UnboundedSender<Rc<Vec<u8>>>>>>,
) {
    let initial_update = tree.borrow().state().serialize();
    let serialized = serde_json::to_vec(&initial_update).unwrap();

    if stream.write_all(&serialized).await.is_err() {
        return;
    }

    let (tree_update_tx, mut tree_update_rx) = mpsc::unbounded();
    tree_update_txs.borrow_mut().push(tree_update_tx.clone());

    let await_disconnect = {
        let mut stream = stream.clone();
        async move {
            let mut buffer = [0u8; 1];
            let _ = stream.read(&mut buffer).await;
        }
    };

    let send_updates = async move {
        while let Some(serialized) = tree_update_rx.next().await {
            if stream.write_all(&serialized).await.is_err() {
                break;
            }
        }
    };

    await_disconnect.or(send_updates).await;

    let mut txs = tree_update_txs.borrow_mut();
    for (i, other_tx) in txs.iter().enumerate() {
        if tree_update_tx.same_receiver(other_tx) {
            txs.remove(i);
            break;
        }
    }
}

fn fd_from_ancillary_buffer(mut buffer: RecvAncillaryBuffer) -> io::Result<OwnedFd> {
    for msg in buffer.drain() {
        if let RecvAncillaryMessage::ScmRights(mut fds) = msg {
            if let Some(fd) = fds.next() {
                return Ok(fd);
            }
        }
    }
    Err(io::ErrorKind::NotFound.into())
}

fn adapter_thread(
    tree: Tree,
    mut action_handler: Box<dyn ActionHandler + Send>,
    tree_request_rx: Async<UnixDatagram>,
    action_request_rx: Async<UnixDatagram>,
    mut tree_update_rx: mpsc::Receiver<TreeUpdate>,
    shutdown_rx: oneshot::Receiver<()>,
) {
    let tree = RefCell::new(tree);
    let ex = LocalExecutor::new();
    let (mut task_tx, mut task_rx) = mpsc::channel::<Task<()>>(0);
    let tree_update_txs = Rc::new(RefCell::new(
        Vec::<mpsc::UnboundedSender<Rc<Vec<u8>>>>::new(),
    ));

    let handle_tasks = async move {
        let mut tasks = Vec::<Task<()>>::new();
        let mut quit = false;

        while !quit {
            (tasks, quit) = match select(task_rx.next(), select_all(tasks)).await {
                Either::Left((result, waiter)) => {
                    if let Some(task) = result {
                        let mut tasks = waiter.into_inner();
                        tasks.push(task);
                        (tasks, false)
                    } else {
                        (waiter.into_inner(), true)
                    }
                }
                Either::Right(((_, _, tasks), _)) => (tasks, false),
            };
        }

        join_all(tasks.into_iter().map(|task| task.cancel())).await;
    };

    let handle_actions = async move {
        let mut buffer = [0u8; 65536];

        loop {
            if let Ok(n) = action_request_rx.recv(&mut buffer).await {
                if let Ok(request) = serde_json::from_slice::<ActionRequest>(&buffer[..n]) {
                    action_handler.do_action(request);
                }
            }
        }
    };

    let handle_tree_updates = async {
        while let Some(update) = tree_update_rx.next().await {
            let serialized = Rc::new(serde_json::to_vec(&update).unwrap());
            tree.borrow_mut().update(update);
            for tx in tree_update_txs.borrow().iter() {
                tx.unbounded_send(Rc::clone(&serialized)).unwrap();
            }
        }
    };

    let handle_tree_requests_and_shutdown = async {
        let mut shutdown_rx = shutdown_rx;

        loop {
            let handle_tree_request = tree_request_rx.read_with(|socket| {
                // The following is largely based on the rcv_msg function
                // in smithay/wayland-rs.
                let mut cmsg_space = vec![0; rustix::cmsg_space!(ScmRights(1))];
                let mut cmsg_buffer = RecvAncillaryBuffer::new(&mut cmsg_space);
                let mut buffer = [0u8, 1];
                let mut iov = [IoSliceMut::new(&mut buffer)];
                recvmsg(
                    socket,
                    &mut iov[..],
                    &mut cmsg_buffer,
                    RecvFlags::DONTWAIT | RecvFlags::CMSG_CLOEXEC,
                )?;

                let fd = fd_from_ancillary_buffer(cmsg_buffer)?;
                let stream = AsyncUnixStream::try_from(fd)?;
                Ok(ex.spawn(tree_stream_task(stream, &tree, Rc::clone(&tree_update_txs))))
            });
            pin!(handle_tree_request);

            shutdown_rx = match select(shutdown_rx, handle_tree_request).await {
                Either::Left(_) => {
                    task_tx.close_channel();
                    break;
                }
                Either::Right((result, shutdown_rx)) => {
                    if let Ok(task) = result {
                        task_tx.send(task).await.unwrap();
                    }
                    shutdown_rx
                }
            };
        }
    };

    let main = handle_tasks
        .or(handle_actions)
        .or(handle_tree_updates)
        .or(handle_tree_requests_and_shutdown);
    block_on(ex.run(main));
}

pub struct Adapter {
    tree_request_fd: OwnedFd,
    action_request_fd: OwnedFd,
    tree_update_tx: RefCell<mpsc::Sender<TreeUpdate>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Adapter {
    pub fn new(
        initial_state: impl 'static + FnOnce() -> TreeUpdate,
        _is_window_focused: bool,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Option<Self> {
        let initial_state = initial_state();
        let tree = Tree::new(initial_state, true);
        let (tree_request_rx, tree_request_tx) = UnixDatagram::pair().unwrap();
        let tree_request_rx = Async::new(tree_request_rx).unwrap();
        let (action_request_rx, action_request_tx) = UnixDatagram::pair().unwrap();
        let action_request_rx = Async::new(action_request_rx).unwrap();
        let (tree_update_tx, tree_update_rx) = mpsc::channel(0);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let thread = std::thread::spawn(move || {
            adapter_thread(
                tree,
                action_handler,
                tree_request_rx,
                action_request_rx,
                tree_update_rx,
                shutdown_rx,
            )
        });
        Some(Self {
            tree_request_fd: tree_request_tx.into(),
            action_request_fd: action_request_tx.into(),
            tree_update_tx: RefCell::new(tree_update_tx),
            shutdown_tx: Some(shutdown_tx),
            thread: Some(thread),
        })
    }

    pub fn set_root_window_bounds(&self, _outer: Rect, _inner: Rect) {
        // backward compat stub
    }

    pub fn update(&self, update: TreeUpdate) {
        let mut tx = self.tree_update_tx.borrow_mut();
        block_on(tx.send(update)).unwrap();
    }

    pub fn update_window_focus_state(&self, _is_focused: bool) {
        // backward compat stub
    }

    pub fn tree_request_fd(&self) -> BorrowedFd<'_> {
        self.tree_request_fd.as_fd()
    }

    pub fn action_request_fd(&self) -> BorrowedFd<'_> {
        self.action_request_fd.as_fd()
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.send(()).unwrap();
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}
