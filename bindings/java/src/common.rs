// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::*;
use jni::{
    objects::{JByteArray, JClass},
    sys::{jint, jlong},
    JNIEnv,
};

use crate::{box_from_jptr, box_str_from_utf8_jbytes, into_jptr, mut_from_jptr, ref_from_jptr};

fn node_id_from_parts(low: jlong, high: jlong) -> NodeId {
    let num = ((high as u128) << 64) | (low as u128);
    NodeId(unsafe { std::num::NonZeroU128::new_unchecked(num) })
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_Node_nativeDrop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    drop(box_from_jptr::<Node>(ptr));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeNew(
    _env: JNIEnv,
    _class: JClass,
    role: jint,
) -> jlong {
    let role = Role::n(role as u8).unwrap();
    let builder = NodeBuilder::new(role);
    into_jptr(builder)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeDrop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    drop(box_from_jptr::<NodeBuilder>(ptr));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeSetName(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    value: JByteArray,
) {
    let builder = mut_from_jptr::<NodeBuilder>(ptr);
    let value = box_str_from_utf8_jbytes(&mut env, value);
    builder.set_name(value);
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_NodeBuilder_nativeBuild(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jlong {
    let builder = box_from_jptr::<NodeBuilder>(ptr);
    let mut classes = NodeClassSet::lock_global();
    let node = builder.build(&mut classes);
    into_jptr(node)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeNew(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let update = TreeUpdate::default();
    into_jptr(update)
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeDrop(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    drop(box_from_jptr::<TreeUpdate>(ptr));
}

#[no_mangle]
pub extern "system" fn Java_dev_accesskit_TreeUpdate_nativeAddNode(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    id_low: jlong,
    id_high: jlong,
    node_ptr: jlong,
) {
    let update = mut_from_jptr::<TreeUpdate>(ptr);
    let id = node_id_from_parts(id_low, id_high);
    let node = box_from_jptr::<Node>(node_ptr);
    update.nodes.push((id, *node));
}
