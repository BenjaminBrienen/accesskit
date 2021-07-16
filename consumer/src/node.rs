// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::iter::FusedIterator;
use std::sync::{Arc, Weak};

use accesskit_schema::{NodeId, Rect, Role};

use crate::tree::{NodeState, ParentAndIndex, Reader as TreeReader, Tree};
use crate::NodeData;

pub struct Node<'a> {
    pub tree_reader: &'a TreeReader<'a>,
    pub(crate) state: &'a NodeState,
}

impl Node<'_> {
    pub fn data(&self) -> &NodeData {
        &self.state.data
    }

    pub fn is_focused(&self) -> bool {
        return self.tree_reader.state.data.focus == Some(self.id());
    }

    pub fn is_ignored(&self) -> bool {
        self.data().ignored || (self.role() == Role::Presentation)
    }

    pub fn is_invisible_or_ignored(&self) -> bool {
        (self.is_invisible() || self.is_ignored()) && !self.is_focused()
    }

    pub fn parent<'a>(&'a self) -> Option<Node<'a>> {
        if let Some(ParentAndIndex(parent, _)) = &self.state.parent_and_index {
            Some(self.tree_reader.node_by_id(*parent).unwrap())
        } else {
            None
        }
    }

    pub fn unignored_parent<'a>(&'a self) -> Option<Node<'a>> {
        if let Some(parent) = self.parent() {
            if parent.is_ignored() {
                // Work around lifetime issues.
                if let Some(result_node) = parent.unignored_parent() {
                    Some(self.tree_reader.node_by_id(result_node.id()).unwrap())
                } else {
                    None
                }
            } else {
                Some(parent)
            }
        } else {
            None
        }
    }

    pub fn children<'a>(
        &'a self,
    ) -> impl DoubleEndedIterator<Item = Node<'a>>
           + ExactSizeIterator<Item = Node<'a>>
           + FusedIterator<Item = Node<'a>>
           + 'a {
        self.data()
            .children
            .iter()
            .map(move |id| self.tree_reader.node_by_id(*id).unwrap())
    }

    // TODO: get unignored children; see Chromium's ui/accessibility/ax_node.cc

    pub fn global_id(&self) -> String {
        format!("{}:{}", self.tree_reader.id().0, self.id().0)
    }

    /// Returns the node's bounds relative to the root of the tree.
    pub fn bounds(&self) -> Option<Rect> {
        if let Some(bounds) = &self.data().bounds {
            // TODO: handle offset container
            assert!(bounds.offset_container.is_none());
            // TODO: handle transform
            assert!(bounds.transform.is_none());
            Some(bounds.rect.clone())
        } else {
            None
        }
    }

    // Convenience getters

    pub fn id(&self) -> NodeId {
        self.data().id
    }

    pub fn role(&self) -> Role {
        self.data().role
    }

    pub fn is_invisible(&self) -> bool {
        self.data().invisible
    }
}

#[derive(Clone)]
pub struct WeakNode {
    pub tree: Weak<Tree>,
    pub id: NodeId,
}

impl WeakNode {
    pub fn map<F, T>(&self, f: F) -> Option<T>
    where
        for<'a> F: FnOnce(&Node<'a>) -> T,
    {
        self.tree
            .upgrade()
            .map(|tree| tree.read().node_by_id(self.id).map(|node| f(&node)))
            .flatten()
    }
}

impl Node<'_> {
    pub fn downgrade(&self) -> WeakNode {
        WeakNode {
            tree: Arc::downgrade(self.tree_reader.tree),
            id: self.id(),
        }
    }
}
