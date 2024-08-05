// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};
use accesskit_windows::{SubclassingAdapter, HWND};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub struct Adapter {
    adapter: SubclassingAdapter,
}

impl Adapter {
    pub fn new(
        _event_loop: &ActiveEventLoop,
        window: &Window,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler + Send,
        _deactivation_handler: impl 'static + DeactivationHandler,
    ) -> Self {
        let hwnd = match window.window_handle().unwrap().as_raw() {
            RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            RawWindowHandle::WinRt(_) => unimplemented!(),
            _ => unreachable!(),
        };

        let adapter = SubclassingAdapter::new(HWND(hwnd), activation_handler, action_handler);
        Self { adapter }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        if let Some(events) = self.adapter.update_if_active(updater) {
            events.raise();
        }
    }

    pub fn process_event(&mut self, _window: &Window, _event: &WindowEvent) {}
}
