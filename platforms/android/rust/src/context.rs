// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::classes::{AccessibilityNodeInfoCompat, ClassCache};
use accesskit::ActionHandler;
use accesskit_consumer::Tree;
use jni::{
    objects::{GlobalRef, JValue},
    sys::jlong,
    JNIEnv,
};
use std::{
    mem::ManuallyDrop,
    sync::{Arc, RwLock, RwLockReadGuard, Weak},
};

pub(crate) struct Context {
    pub(crate) tree: RwLock<Tree>,
    pub(crate) action_handler: Box<dyn ActionHandler + Send + Sync>,
    weak_ref: ManuallyDrop<Weak<Context>>,
    pub(crate) node_info_class: AccessibilityNodeInfoCompat,
}

impl Context {
    pub(crate) fn new(
        env: &mut JNIEnv<'_>,
        tree: Tree,
        action_handler: Box<dyn ActionHandler + Send + Sync>,
        class_cache: &ClassCache,
        delegate: &GlobalRef,
    ) -> Arc<Self> {
        Arc::new_cyclic(|weak_ref| {
            class_cache
                .delegate
                .set_context(env, delegate, JValue::Long(weak_ref.as_ptr() as jlong))
                .unwrap();
            Self {
                tree: RwLock::new(tree),
                action_handler,
                weak_ref: ManuallyDrop::new(weak_ref.clone()),
                node_info_class: class_cache.node_info.clone(),
            }
        })
    }

    pub(crate) unsafe fn from_jni(ptr: jlong) -> ManuallyDrop<Weak<Self>> {
        ManuallyDrop::new(Weak::from_raw(ptr as *mut Context))
    }

    pub(crate) fn read_tree(&self) -> RwLockReadGuard<'_, Tree> {
        self.tree.read().unwrap()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.weak_ref) };
    }
}
