use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
};

use hashbrown::HashMap;
use html5ever::QualName;

pub(super) type NodeId = u64;

#[derive(Debug)]
pub enum MemberKind {
    Root,
    Element { name: QualName },
    Comment { content: String },
    Text { contents: String },
}

#[derive(Debug)]
struct DomEntry {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub myself: MemberKind,
    pub children: VecDeque<NodeId>,
}

#[derive(Default, Debug)]
pub struct DomTree {
    entries: HashMap<NodeId, DomEntry>,
}

impl DomTree {
    pub fn add_node(&mut self, new_member: MemberKind) -> NodeId {
        static NEXT_AVAILABLE_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_AVAILABLE_ID.fetch_add(1, Ordering::Relaxed);
        self.entries.entry(id).or_insert_with(|| DomEntry {
            myself: new_member,
            id,
            parent: None,
            children: Default::default(),
        });
        id
    }

    fn node(&self, node_id: NodeId) -> &DomEntry {
        self.entries
            .get(&node_id)
            .expect("Could not find expected member")
    }

    pub fn parent_of(&self, node_id: NodeId) -> Option<NodeId> {
        self.entries.get(&node_id)?.parent
    }

    pub fn prepend_to(&mut self, parent: NodeId, child: NodeId) {
        dbg!(&self.entries);
        dbg!(parent);
        dbg!(child);
        let [parent_entry, child_entry] = self
            .entries
            .get_many_mut([&parent, &child])
            .expect("Could not find parent and/or child in DOM");

        child_entry.parent = Some(parent);
        parent_entry.children.push_front(child);
    }

    pub fn append_to(&mut self, parent: NodeId, child: NodeId) {
        dbg!(&self.entries);
        dbg!(parent);
        dbg!(child);
        let [parent_entry, child_entry] = self
            .entries
            .get_many_mut([&parent, &child])
            .expect("Could not find parent and/or child in DOM");

        child_entry.parent = Some(parent);
        parent_entry.children.push_back(child);
    }

    pub fn insert_before(&mut self, parent: NodeId, sibling: NodeId, child: NodeId) {
        let [parent_entry, child_entry] = self
            .entries
            .get_many_mut([&parent, &child])
            .expect("Could not find parent and/or child in DOM");

        let sibling_position = parent_entry
            .children
            .iter()
            .position(|&id| id == sibling)
            .expect("Could not find sibling attached to parent");

        child_entry.parent = Some(parent);
        parent_entry.children.insert(sibling_position, child);
    }

    pub fn insert_after(&mut self, parent: NodeId, sibling: NodeId, child: NodeId) {
        let [parent_entry, child_entry] = self
            .entries
            .get_many_mut([&parent, &child])
            .expect("Could not find parent and/or child in DOM");

        let sibling_position = parent_entry
            .children
            .iter()
            .position(|&id| id == sibling)
            .expect("Could not find sibling attached to parent");

        child_entry.parent = Some(parent);

        if sibling_position == parent_entry.children.len() {
            parent_entry.children.push_back(child);
        } else {
            parent_entry.children.insert(sibling_position, child);
        }
    }
}
