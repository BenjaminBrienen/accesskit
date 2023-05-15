// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{
    Action, ActionRequest, CheckedState, NodeId, NodeIdContent, Role,
};
use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState, TreeState};
use crate::{context::Context, util::*};
use jni::{
    errors::Result,
    objects::{JClass, JObject, JValue},
    sys::{jint, jlong, jobject},
    JNIEnv,
};
use std::{mem::ManuallyDrop, sync::{Arc, Weak}};

fn filter_common(node: &NodeState) -> FilterResult {
    if node.is_hidden() {
        return FilterResult::ExcludeSubtree;
    }

    let role = node.role();
    if role == Role::Presentation || role == Role::GenericContainer || role == Role::InlineTextBox {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

pub(crate) fn filter(node: &Node) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }

    filter_common(node.state())
}

pub(crate) enum NodeWrapper<'a> {
    Node(&'a Node<'a>),
    DetachedNode(&'a DetachedNode),
}

impl<'a> NodeWrapper<'a> {
    fn node_state(&self) -> &'a NodeState {
        match self {
            Self::Node(node) => node.state(),
            Self::DetachedNode(node) => node.state(),
        }
    }

    fn name(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
    }

    fn is_enabled(&self) -> bool {
        !self.node_state().is_disabled()
    }

    fn is_focusable(&self) -> bool {
        self.node_state().is_focusable()
    }

    fn is_focused(&self) -> bool {
        match self {
            Self::Node(node) => node.is_focused(),
            Self::DetachedNode(node) => node.is_focused(),
        }
    }

    fn is_checkable(&self) -> bool {
        self.node_state().checked_state().is_some()
    }

    fn is_checked(&self) -> bool {
        match self.node_state().checked_state().unwrap() {
            CheckedState::False => false,
            CheckedState::True => true,
            CheckedState::Mixed => true,
        }
    }

    fn is_selected(&self) -> bool {
        match self.node_state().role() {
            // https://www.w3.org/TR/core-aam-1.1/#mapping_state-property_table
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-checked for 'radio' and 'menuitemradio' roles.
            Role::RadioButton | Role::MenuItemRadio => {
                self.node_state().checked_state() == Some(CheckedState::True)
            }
            // https://www.w3.org/TR/wai-aria-1.1/#aria-selected
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-selected.
            _ => self.node_state().is_selected().unwrap_or(false),
        }
    }
}

const HOST_VIEW_ID: jint = -1;

pub(crate) enum PlatformNodeId {
    Root,
    Resolved(NodeId),
}

impl PlatformNodeId {
    fn from_jni(id: jint) -> Self {
        if id == HOST_VIEW_ID {
            Self::Root
        } else {
            Self::Resolved(NodeId(unsafe { NodeIdContent::new_unchecked(id as u128) }))
        }
    }
    
    fn resolve(&self, tree: &TreeState) -> NodeId {
        match self {
            Self::Root => tree.root_id(),
            Self::Resolved(id) => *id,
        }
    }
}

pub(crate) struct PlatformNode {
    comes_from_jni: bool,
    pub(crate) context: ManuallyDrop<Weak<Context>>,
    pub(crate) node_id: PlatformNodeId,
}

impl PlatformNode {
    pub(crate) fn new(context: &Arc<Context>, node_id: NodeId) -> Self {
        Self {
            comes_from_jni: false,
            context: ManuallyDrop::new(Arc::downgrade(context)),
            node_id: PlatformNodeId::Resolved(node_id),
        }
    }
    
    unsafe fn from_jni(context: jlong, node_id: jint) -> Self {
        Self {
            comes_from_jni: true,
            context: Context::from_jni(context),
            node_id: PlatformNodeId::from_jni(node_id),
        }
    }

    fn upgrade_context(&self) -> Result<Arc<Context>> {
        upgrade(&self.context)
    }
    
    fn with_tree_state_and_context<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState, &Context) -> Result<T>,
    {
        let context = self.upgrade_context()?;
        let tree = context.read_tree();
        f(tree.state(), &context)
    }

    fn with_tree_state<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState) -> Result<T>,
    {
        self.with_tree_state_and_context(|state, _| f(state))
    }

    fn resolve_with_context<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(Node<'a>, &Context) -> Result<T>,
    {
        self.with_tree_state_and_context(|state, context| {
            let node_id = self.node_id.resolve(state);
            if let Some(node) = state.node_by_id(node_id) {
                f(node, context)
            } else {
                Err(node_not_found())
            }
        })
    }

    fn do_action<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(NodeId) -> ActionRequest,
    {
        let context = self.upgrade_context()?;
        let tree = context.read_tree();
        let node_id = self.node_id.resolve(tree.state());
        if tree.state().has_node(node_id) {
            drop(tree);
            let request = f(node_id);
            context.action_handler.do_action(request);
            Ok(())
        } else {
            Err(node_not_found())
        }
    }

    fn do_default_action(&self) -> Result<()> {
        self.do_action(|target| ActionRequest {
            action: Action::Default,
            target,
            data: None,
        })
    }
}

impl Drop for PlatformNode {
    fn drop(&mut self) {
        if !self.comes_from_jni {
            unsafe { ManuallyDrop::drop(&mut self.context) };
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_accesskit_AccessKit_AccessibilityDelegate_populateAccessibilityNodeInfo<
    'local,
>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    context: jlong,
    host: JObject<'local>,
    jni_node: JObject<'local>,
    virtual_view_id: jint,
) -> jobject {
    let platform_node = unsafe { PlatformNode::from_jni(context, virtual_view_id) };
    platform_node.resolve_with_context(|resolved_node, context| {
        let node_info_class = &context.node_info_class;
        
        for child in resolved_node.filtered_children(&filter) {
            node_info_class.addChild(&mut env, &jni_node, object_value(&host), id_value(child.id()))?;
        }
        if let Some(parent) = resolved_node.filtered_parent(&filter) {
            if !parent.is_root() {
                node_info_class.setParent(&mut env, &jni_node, object_value(&host), id_value(parent.id()))?;
            }
        }
        
        let wrapper = NodeWrapper::Node(&resolved_node);
        if wrapper.is_checkable() {
            node_info_class.setCheckable(&mut env, &jni_node, bool_value(true))?;
            node_info_class.setChecked(&mut env, &jni_node, bool_value(wrapper.is_checked()))?;
        }
        node_info_class.setEnabled(&mut env, &jni_node, bool_value(wrapper.is_enabled()))?;
        node_info_class.setFocusable(&mut env, &jni_node, bool_value(wrapper.is_focusable()))?;
        node_info_class.setFocused(&mut env, &jni_node, bool_value(wrapper.is_focused()))?;
        node_info_class.setPassword(&mut env, &jni_node, bool_value(wrapper.node_state().is_protected()))?;
        node_info_class.setSelected(&mut env, &jni_node, bool_value(wrapper.is_selected()))?;
        if let Some(name) = wrapper.name() {
            let name = env.new_string(name)?;
            node_info_class.setText(&mut env, &jni_node, JValue::Object(&name).as_jni())?;
        }
        
        Ok(())
    }).unwrap();
    jni_node.into_raw()
}
