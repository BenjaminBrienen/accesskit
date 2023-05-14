// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::{
    classes::{ClassCache, CLASS_CACHE},
    context::Context,
};
use accesskit::{ActionHandler, TreeUpdate};
use accesskit_consumer::Tree;
use jni::{
    errors::Result,
    objects::{JClass, JObject},
    sys::{jboolean, jint, jlong, jobject, JNI_FALSE, JNI_TRUE},
    JNIEnv,
};
use std::sync::Arc;

pub struct Adapter {
    context: Arc<Context>,
}

impl Adapter {
    pub unsafe fn new(
        env: *mut jni::sys::JNIEnv,
        activity: jobject,
        initial_state: TreeUpdate,
        action_handler: Box<dyn ActionHandler + Send + Sync>,
    ) -> Result<Self> {
        let mut env = JNIEnv::from_raw(env)?;
        let activity = JObject::from_raw(activity);
        let class_cache = CLASS_CACHE.read().unwrap();
        let class_cache_ref = class_cache.as_ref().unwrap();
        let delegate_instance = class_cache_ref.delegate.create_new_instance(&mut env)?;
        let context = Context::new(
            &mut env,
            Tree::new(initial_state),
            action_handler,
            class_cache_ref,
            &delegate_instance,
        );
        class_cache_ref
            .delegate
            .install(&mut env, delegate_instance, &activity)?;
        Ok(Self { context })
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_accesskit_AccessKit_Adapter_initialize<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
) -> jboolean {
    let mut class_cache = CLASS_CACHE.write().unwrap();
    if class_cache.is_some() {
        return JNI_TRUE;
    }
    match ClassCache::new(&mut env) {
        Ok(cache) => {
            *class_cache = Some(cache);
            JNI_TRUE
        }
        _ => JNI_FALSE,
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_accesskit_AccessKit_AccessibilityDelegate_populateAccessibilityNodeInfo<
    'local,
>(
    env: JNIEnv<'local>,
    _: JClass<'local>,
    context: jlong,
    host: JObject<'local>,
    node: JObject<'local>,
    virtual_view_id: jint,
) -> jobject {
    let context = unsafe { Context::from_jni(context) };

    node.into_raw()
}
