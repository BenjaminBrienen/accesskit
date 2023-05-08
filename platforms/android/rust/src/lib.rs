use jni::{
    objects::{GlobalRef, JClass, JFieldID, JMethodID},
    sys::{jboolean, JNI_FALSE, JNI_TRUE},
    JNIEnv,
};
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};

struct JniCache {
    delegate_class_id: GlobalRef,
    delegate_constructor_id: JMethodID,
    delegate_ptr_id: JFieldID,
}

lazy_static! {
    static ref JNI_CACHE: Arc<RwLock<Option<JniCache>>> = Arc::new(RwLock::new(None));
}

#[no_mangle]
pub extern "C" fn Java_dev_accesskit_AccessKit_Adapter_initialize<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
) -> jboolean {
    let mut cache = JNI_CACHE.write().unwrap();
    if cache.is_some() {
        return JNI_FALSE;
    }
    let delegate_class_id = match env.find_class("dev/accesskit/AccessKit/AccessibilityDelegate") {
        Ok(class) => match env.new_global_ref(class) {
            Ok(class) => class,
            _ => return JNI_FALSE,
        },
        _ => return JNI_FALSE,
    };
    let delegate_constructor_id = match env.get_method_id(&delegate_class_id, "<init>", "V") {
        Ok(constructor) => constructor,
        _ => return JNI_FALSE,
    };
    let delegate_ptr_id = match env.get_field_id(&delegate_class_id, "ptr", "L") {
        Ok(field) => field,
        _ => return JNI_FALSE,
    };
    *cache = Some(JniCache {
        delegate_class_id,
        delegate_constructor_id,
        delegate_ptr_id,
    });
    JNI_TRUE
}
