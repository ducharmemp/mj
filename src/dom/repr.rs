use std::collections::{HashMap, VecDeque};

use html5ever::QualName;

pub(super) type NodeId = u64;

pub enum MemberKind {
    Root,
    Element { name: QualName },
    Comment { content: String },
    Text { contents: String },
}

struct DomEntry {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub myself: MemberKind,
    pub children: VecDeque<NodeId>,
}

#[derive(Default)]
pub struct DomTree {
    next_available_id: NodeId,
    entries: HashMap<NodeId, DomEntry>,
}
impl DomTree {
    pub fn add_node(&mut self, new_member: MemberKind) -> NodeId {
        let id = self.next_available_id;
        self.next_available_id += 1;
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
        let parent_entry = self
            .entries
            .get_mut(&parent)
            .expect("Could not find parent in DOM");

        parent_entry.children.push_front(child);
    }

    pub fn append_to(&mut self, parent: NodeId, child: NodeId) {
        let parent_entry = self
            .entries
            .get_mut(&parent)
            .expect("Could not find parent in DOM");

        parent_entry.children.push_back(child);
    }

    pub fn insert_before(&mut self, parent: NodeId, sibling: NodeId, child: NodeId) {
        let parent_entry = self
            .entries
            .get_mut(&parent)
            .expect("Could not find parent in DOM");

        let sibling_position = parent_entry
            .children
            .iter()
            .position(|&id| id == sibling)
            .expect("Could not find sibling attached to parent");
        parent_entry.children.insert(sibling_position, child);
    }

    pub fn insert_after(&mut self, parent: NodeId, sibling: NodeId, child: NodeId) {
        let parent_entry = self
            .entries
            .get_mut(&parent)
            .expect("Could not find parent in DOM");

        let sibling_position = parent_entry
            .children
            .iter()
            .position(|&id| id == sibling)
            .expect("Could not find sibling attached to parent");

        if sibling_position == parent_entry.children.len() {
            parent_entry.children.push_back(child);
        } else {
            parent_entry.children.insert(sibling_position, child);
        }
    }
}
