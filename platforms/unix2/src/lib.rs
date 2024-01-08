// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from smithay-clipboard.
// Copyright (c) 2018 Lucas Timmins & Victor Berger
// Licensed under the MIT license (found in the LICENSE-MIT file).

use accesskit::{ActionHandler, Rect, TreeUpdate};
use sctk::reexports::{
    calloop::channel::{self, Sender},
    client::{
        backend::{Backend, ObjectId},
        protocol::{__interfaces::WL_SURFACE_INTERFACE, wl_surface::WlSurface},
        Connection, Proxy,
    },
};
use std::{
    collections::HashMap,
    ffi::c_void,
    os::unix::io::AsFd,
    sync::{Arc, Mutex},
};
use wayland_protocols::wp::accessibility::v1::client::wp_accessibility_provider_v1::WpAccessibilityProviderV1;

mod state;
mod worker;

pub struct Adapter {
    surface: WlSurface,
    request_tx: Sender<worker::Command>,
    instances: Arc<Mutex<HashMap<u32, WpAccessibilityProviderV1>>>,
    worker_thread: Option<std::thread::JoinHandle<()>>,
}

impl Adapter {
    /// Creates an AccessKit adapter for the specified Wayland display
    /// and surface. The adapter will run on a worker thread with its own
    /// libwayland event queue. Both the tree source and the action handler
    /// will always be called on that worker thread.
    ///
    /// # Safety
    ///
    /// `display` must be a valid `*mut wl_display` pointer, and
    /// `surface` must be a valid `*mut wl_surface` pointer. Both must remain
    /// valid for as long as the adapter is alive.
    pub unsafe fn new(
        display: *mut c_void,
        surface: *mut c_void,
        source: impl 'static + FnOnce() -> TreeUpdate + Send,
        action_handler: Box<dyn ActionHandler + Send>,
    ) -> Self {
        let backend = unsafe { Backend::from_foreign_display(display.cast()) };
        let connection = Connection::from_backend(backend);
        let surface_id =
            unsafe { ObjectId::from_ptr(&WL_SURFACE_INTERFACE, surface.cast()) }.unwrap();
        let surface = WlSurface::from_id(&connection, surface_id).unwrap();
        let (request_tx, request_rx) = channel::channel();
        let instances = Arc::new(Mutex::new(HashMap::new()));
        let worker_thread = worker::spawn(
            connection,
            surface.clone(),
            source,
            action_handler,
            request_rx,
            Arc::clone(&instances),
        );

        Self {
            surface,
            request_tx,
            instances,
            worker_thread,
        }
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    pub fn update_if_active(&self, update_factory: impl FnOnce() -> TreeUpdate) {
        use rustix::pipe::{pipe_with, PipeFlags};

        let instances = self.instances.lock().unwrap();
        if instances.is_empty() {
            return;
        }
        let update = update_factory();
        let serialized = Arc::new(serde_json::to_vec(&update).unwrap());
        self.request_tx
            .send(worker::Command::UpdateTree(update))
            .unwrap();
        for instance in instances.values() {
            let (read_fd, write_fd) = pipe_with(PipeFlags::CLOEXEC).unwrap();
            self.request_tx
                .send(worker::Command::WriteUpdate(
                    write_fd,
                    Arc::clone(&serialized),
                ))
                .unwrap();
            instance.update(&self.surface, read_fd.as_fd());
        }
    }

    pub fn update_window_focus_state(&self, _is_focused: bool) {
        // stub for backward compat
    }

    pub fn set_root_window_bounds(&self, _outer: Rect, _inner: Rect) {
        // stub for backward compat
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        let _ = self.request_tx.send(worker::Command::Exit);
        if let Some(worker_thread) = self.worker_thread.take() {
            let _ = worker_thread.join();
        }
    }
}
