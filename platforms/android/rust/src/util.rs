// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::NodeId;
use jni::{errors::{Error, JniError, Result}, objects::JObject, sys::jvalue};
use std::sync::{Arc, Weak};

pub(crate) fn bool_value(value: bool) -> jvalue {
    jvalue { z: value as u8 }
}

pub(crate) fn id_value(value: NodeId) -> jvalue {
    jvalue { i: value.0.get() as i32 }
}

pub(crate) fn object_value<'local, O>(value: O) -> jvalue
where O: AsRef<JObject<'local>>
{
    jvalue { l: value.as_ref().as_raw() }
}

pub(crate) fn node_not_found() -> Error {
    Error::JniCall(JniError::InvalidArguments)
}

pub(crate) fn upgrade<T>(weak: &Weak<T>) -> Result<Arc<T>> {
    if let Some(strong) = weak.upgrade() {
        Ok(strong)
    } else {
        Err(node_not_found())
    }
}
