// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use jni::{
    errors::Result,
    objects::{GlobalRef, JObject, JValue},
    signature::{Primitive, ReturnType},
    JNIEnv,
};
use std::sync::{Arc, RwLock};

macro_rules! java_class {
    (
        package $package_name:literal;
        class $class_name:ident {
        $(field $field_type:literal $field_name:ident;)*
        $(ctor $constructor_name:ident($($constructor_arg_type:literal $constructor_arg_name:ident,)*);)*
        $(method $method_return_type:literal $method_name:ident($($method_arg_type:literal $method_arg_name:ident,)*);)*}
    ) => {
        paste::paste! {
            #[derive(Clone)]
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            pub(crate) struct $class_name {
                class: jni::objects::GlobalRef,
                $([<$field_name _id>]: jni::objects::JFieldID,)*
                $([<$constructor_name _id>]: jni::objects::JMethodID,)*
                $([<$method_name _id>]: jni::objects::JMethodID,)*
            }
            #[allow(non_snake_case)]
            impl $class_name {
                pub(crate) fn initialize_class(env: &mut jni::JNIEnv<'_>) -> jni::errors::Result<Self> {
                    let class = env.find_class(concat!($package_name, "/", stringify!($class_name)))?;
                    Ok(Self {
                        class: env.new_global_ref(&class)?,
                        $([<$field_name _id>]: env.get_field_id(&class, stringify!($field_name), $field_type)?,)*
                        $([<$constructor_name _id>]: env.get_method_id(
                            &class,
                            "<init>",
                            concat!("(", $($constructor_arg_type,)* ")V"),
                        )?,)*
                        $([<$method_name _id>]: env.get_method_id(
                            &class,
                            stringify!($method_name),
                            concat!("(", $($method_arg_type,)* ")", $method_return_type),
                        )?,)*
                    })
                }
                $(pub(crate) fn $constructor_name<'local>(
                    &self,
                    env: &mut jni::JNIEnv<'local>,
                    $($constructor_arg_name: jni::sys::jvalue,)*
                ) -> jni::errors::Result<jni::objects::JObject<'local>> {
                    unsafe {
                        let class = jni::objects::JClass::from_raw(self.class.as_raw());
                        env.new_object_unchecked(
                            class,
                            self.[<$constructor_name _id>],
                            &[$($constructor_arg_name,)*]
                        )
                    }
                })*
                $(#[inline]
                pub(crate) fn [<set_ $field_name>]<'local, 'other_local, O>(
                    &self,
                    env: &mut jni::JNIEnv<'local>,
                    instance: O,
                    value: jni::objects::JValue,
                ) -> jni::errors::Result<()>
                where O: AsRef<jni::objects::JObject<'other_local>>
                {
                    env.set_field_unchecked(instance, self.[<$field_name _id>], value)
                })*
                $(#[inline]
                pub(crate) fn $method_name<'local, 'other_local, O>(
                    &self,
                    env: &mut jni::JNIEnv<'local>,
                    instance: O,
                    $($method_arg_name: jni::sys::jvalue,)*
                ) -> jni::errors::Result<jni::objects::JValueOwned<'local>>
                where O: AsRef<jni::objects::JObject<'other_local>>
                {
                    unsafe {
                        env.call_method_unchecked(
                            instance,
                            self.[<$method_name _id>],
                            return_type!($method_return_type),
                            &[$($method_arg_name),*]
                        )
                    }
                })*
            }
        }
    }
}

macro_rules! return_type {
    ("V") => {
        jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void)
    };
}

java_class! {
    package "dev/accesskit/AccessKit";

    class AccessibilityDelegate {
        field "J" context;

        ctor constructor();
    }
}

impl AccessibilityDelegate {
    pub(crate) fn create_new_instance(&self, env: &mut JNIEnv<'_>) -> Result<GlobalRef> {
        let instance = self.constructor(env)?;
        env.new_global_ref(instance)
    }

    pub(crate) fn install<'local>(
        &self,
        env: &mut JNIEnv<'local>,
        instance: GlobalRef,
        activity: &JObject<'local>,
    ) -> Result<()> {
        let surface_view = env
            .get_field(
                activity,
                "mSurfaceView",
                "Lcom/google/androidgamesdk/GameActivity$InputEnabledSurfaceView;",
            )?
            .l()?;
        let view_class = env.find_class("android/view/View")?;
        let set_delegate_method = env.get_method_id(
            view_class,
            "setAccessibilityDelegate",
            "(Landroid/view/View$AccessibilityDelegate;)V",
        )?;
        unsafe {
            env.call_method_unchecked(
                surface_view,
                set_delegate_method,
                ReturnType::Primitive(Primitive::Void),
                &[JValue::Object(&instance).as_jni()],
            )
        }?;
        Ok(())
    }
}

java_class! {
    package "androidx/core/view/accessibility";

    class AccessibilityNodeInfoCompat {
        method "V" addChild("Landroid/view/View;" view, "I" virtual_descendant_id,);
        method "V" setCheckable("Z" checkable,);
        method "V" setChecked("Z" checked,);
        method "V" setEnabled("Z" enabled,);
        method "V" setFocusable("Z" focusable,);
        method "V" setFocused("Z" focused,);
        method "V" setParent("Landroid/view/View;" view, "I" virtual_descendant_id,);
        method "V" setPassword("Z" password,);
        method "V" setSelected("Z" selected,);
        method "V" setText("Ljava/lang/CharSequence;" text,);
    }
}

pub(crate) struct ClassCache {
    pub(crate) delegate: AccessibilityDelegate,
    pub(crate) node_info: AccessibilityNodeInfoCompat,
}

impl ClassCache {
    pub(crate) fn new(env: &mut JNIEnv<'_>) -> Result<Self> {
        Ok(Self {
            delegate: AccessibilityDelegate::initialize_class(env)?,
            node_info: AccessibilityNodeInfoCompat::initialize_class(env)?,
        })
    }
}

lazy_static::lazy_static! {
    pub(crate) static ref CLASS_CACHE: Arc<RwLock<Option<ClassCache>>> = Arc::new(RwLock::new(None));
}
