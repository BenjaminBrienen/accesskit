// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from smithay-clipboard.
// Copyright (c) 2018 Lucas Timmins & Victor Berger
// Licensed under the MIT license (found in the LICENSE-MIT file).

use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::Tree;
use once_cell::unsync::Lazy;
use sctk::{
    data_device_manager::{ReadPipe, WritePipe},
    delegate_registry,
    reexports::{
        calloop::{LoopHandle, PostAction},
        client::{
            globals::GlobalList, protocol::wl_surface::WlSurface, Connection, Dispatch, QueueHandle,
        },
    },
    registry::{ProvidesRegistryState, RegistryHandler, RegistryState},
    registry_handlers,
};
use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    os::unix::io::{AsFd, AsRawFd, OwnedFd, RawFd},
    sync::{Arc, Mutex},
};
use wayland_protocols::wp::accessibility::v1::client::wp_accessibility_provider_v1::{
    Event, WpAccessibilityProviderV1,
};

type LazyTree = Lazy<Tree, Box<dyn FnOnce() -> Tree>>;

pub(crate) struct State {
    registry_state: RegistryState,
    loop_handle: LoopHandle<'static, Self>,
    pub(crate) exit: bool,
    surface: WlSurface,
    tree: LazyTree,
    action_handler: Box<dyn ActionHandler + Send>,
    instances: Arc<Mutex<HashMap<u32, WpAccessibilityProviderV1>>>,
}

impl State {
    pub(crate) fn new(
        globals: &GlobalList,
        loop_handle: LoopHandle<'static, Self>,
        surface: WlSurface,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: Box<dyn ActionHandler + Send>,
        instances: Arc<Mutex<HashMap<u32, WpAccessibilityProviderV1>>>,
    ) -> Self {
        let tree: LazyTree = Lazy::new(Box::new(move || Tree::new(source(), true)));

        Self {
            registry_state: RegistryState::new(globals),
            loop_handle,
            exit: false,
            surface,
            tree,
            action_handler,
            instances,
        }
    }

    pub(crate) fn update_tree(&mut self, update: TreeUpdate) {
        if let Some(tree) = Lazy::get_mut(&mut self.tree) {
            tree.update(update);
        }
    }

    pub(crate) fn write_update(&self, fd: OwnedFd, serialized: Arc<Vec<u8>>) {
        let write_pipe = WritePipe::from(fd);
        unsafe {
            if set_non_blocking(write_pipe.as_raw_fd()).is_err() {
                return;
            }
        }
        let mut written = 0;
        let _ = self
            .loop_handle
            .insert_source(write_pipe, move |_, file, _| {
                let file = unsafe { file.get_mut() };
                loop {
                    match file.write(&serialized[written..]) {
                        Ok(n) if written + n == serialized.len() => {
                            written += n;
                            break PostAction::Remove;
                        }
                        Ok(n) => written += n,
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            break PostAction::Continue
                        }
                        Err(_) => break PostAction::Remove,
                    }
                }
            });
    }

    fn handle_action_request(&mut self, fd: OwnedFd) {
        let read_pipe = ReadPipe::from(fd);
        unsafe {
            if set_non_blocking(read_pipe.as_raw_fd()).is_err() {
                return;
            }
        }
        let mut reader_buffer = [0; 4096];
        let mut content = Vec::new();
        let _ = self
            .loop_handle
            .insert_source(read_pipe, move |_, file, state| {
                let file = unsafe { file.get_mut() };
                loop {
                    match file.read(&mut reader_buffer) {
                        Ok(0) => {
                            let request = match serde_json::from_slice(&content) {
                                Ok(request) => request,
                                Err(_) => {
                                    break PostAction::Remove;
                                }
                            };
                            state.action_handler.do_action(request);
                            break PostAction::Remove;
                        }
                        Ok(n) => content.extend_from_slice(&reader_buffer[..n]),
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            break PostAction::Continue
                        }
                        Err(_) => {
                            break PostAction::Remove;
                        }
                    };
                }
            });
    }
}

impl ProvidesRegistryState for State {
    registry_handlers![State];

    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}

impl Dispatch<WpAccessibilityProviderV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &WpAccessibilityProviderV1,
        event: Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        if let Event::ActionRequest { surface, fd } = event {
            if surface != state.surface {
                return;
            }
            state.handle_action_request(fd);
        }
    }
}

impl RegistryHandler<State> for State {
    fn new_global(
        state: &mut Self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        name: u32,
        interface: &str,
        _: u32,
    ) {
        use rustix::pipe::{pipe_with, PipeFlags};

        if interface != "wp_accessibility_provider_v1" {
            return;
        }
        let instance = state.registry().bind_specific(qh, name, 1..=1, ()).unwrap();
        let mut instances = state.instances.lock().unwrap();
        let tree = Lazy::force(&state.tree);
        let update = tree.state().serialize();
        let serialized = Arc::new(serde_json::to_vec(&update).unwrap());
        let (read_fd, write_fd) = pipe_with(PipeFlags::CLOEXEC).unwrap();
        state.write_update(write_fd, serialized);
        instance.update(&state.surface, read_fd.as_fd());
        state.surface.commit();
        instances.insert(name, instance);
    }

    fn remove_global(
        state: &mut Self,
        _: &Connection,
        _: &QueueHandle<Self>,
        name: u32,
        interface: &str,
    ) {
        if interface != "wp_accessibility_provider_v1" {
            return;
        }
        state.instances.lock().unwrap().remove(&name);
    }
}

delegate_registry!(State);

unsafe fn set_non_blocking(raw_fd: RawFd) -> std::io::Result<()> {
    let flags = libc::fcntl(raw_fd, libc::F_GETFL);

    if flags < 0 {
        return Err(std::io::Error::last_os_error());
    }

    let result = libc::fcntl(raw_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    if result < 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
}
