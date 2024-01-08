// Copyright 2024 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from smithay-clipboard.
// Copyright (c) 2018 Lucas Timmins & Victor Berger
// Licensed under the MIT license (found in the LICENSE-MIT file).

use accesskit::{ActionHandler, TreeUpdate};
use sctk::reexports::{
    calloop::{
        channel::{self, Channel},
        EventLoop,
    },
    calloop_wayland_source::WaylandSource,
    client::{globals::registry_queue_init, protocol::wl_surface::WlSurface, Connection},
};
use std::{
    collections::HashMap,
    os::unix::io::OwnedFd,
    sync::{Arc, Mutex},
};
use wayland_protocols::wp::accessibility::v1::client::wp_accessibility_provider_v1::WpAccessibilityProviderV1;

use crate::state::State;

pub(crate) fn spawn(
    connection: Connection,
    surface: WlSurface,
    source: impl 'static + FnOnce() -> TreeUpdate + Send,
    action_handler: Box<dyn ActionHandler + Send>,
    request_rx: Channel<Command>,
    instances: Arc<Mutex<HashMap<u32, WpAccessibilityProviderV1>>>,
) -> Option<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("accesskit-adapter".into())
        .spawn(move || {
            worker_impl(
                connection,
                surface,
                source,
                action_handler,
                request_rx,
                instances,
            );
        })
        .ok()
}

pub(crate) enum Command {
    UpdateTree(TreeUpdate),
    WriteUpdate(OwnedFd, Arc<Vec<u8>>),
    Exit,
}

fn worker_impl(
    connection: Connection,
    surface: WlSurface,
    source: impl 'static + FnOnce() -> TreeUpdate + Send,
    action_handler: Box<dyn ActionHandler + Send>,
    request_rx: Channel<Command>,
    instances: Arc<Mutex<HashMap<u32, WpAccessibilityProviderV1>>>,
) {
    let (globals, event_queue) = match registry_queue_init(&connection) {
        Ok(data) => data,
        Err(_) => return,
    };

    let mut event_loop = EventLoop::<State>::try_new().unwrap();
    let loop_handle = event_loop.handle();

    let mut state = State::new(
        &globals,
        loop_handle.clone(),
        surface,
        source,
        action_handler,
        instances,
    );

    loop_handle
        .insert_source(request_rx, |event, _, state| {
            if let channel::Event::Msg(event) = event {
                match event {
                    Command::UpdateTree(update) => state.update_tree(update),
                    Command::WriteUpdate(fd, serialized) => state.write_update(fd, serialized),
                    Command::Exit => state.exit = true,
                }
            }
        })
        .unwrap();

    WaylandSource::new(connection, event_queue)
        .insert(loop_handle)
        .unwrap();

    loop {
        event_loop.dispatch(None, &mut state).unwrap();

        if state.exit {
            break;
        }
    }
}
