use std::collections::VecDeque;

use html5ever::QualName;
use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct NodeId; }

#[derive(Debug)]
pub enum MemberKind {
    Root,
    Element { name: QualName },
    Comment { content: String },
    Text { contents: String },
}

#[derive(Debug)]
pub struct DomEntry {
    pub parent: Option<NodeId>,
    pub myself: MemberKind,
    pub children: VecDeque<NodeId>,
}

#[derive(Default, Debug)]
pub struct DomTree {
    root_id: NodeId,
    entries: SlotMap<NodeId, DomEntry>,
}

impl DomTree {
    pub fn new() -> (Self, NodeId) {
        let mut t = Self::default();
        let root_id = t.add_node(MemberKind::Root);
        t.root_id = root_id;
        (t, root_id)
    }

    pub fn root(&self) -> &DomEntry {
        self.entries
            .get(self.root_id)
            .expect("No root found in the DOM")
    }

    pub fn html(&self) -> Option<&DomEntry> {
        self.find_child(self.root(), "html")
    }

    pub fn head(&self) -> Option<&DomEntry> {
        self.html()
            .and_then(|parent| self.find_child(parent, "head"))
    }

    pub fn body(&self) -> Option<&DomEntry> {
        self.html()
            .and_then(|parent| self.find_child(parent, "body"))
    }

    fn find_child(&self, parent: &DomEntry, tag_name: &str) -> Option<&DomEntry> {
        let mut direct_elements = parent.children.iter().map(|child| {
            self.entries
                .get(*child)
                .expect("Could not find child in DOM")
        });
        direct_elements.find(|child| {
            if let MemberKind::Element { name, .. } = &child.myself {
                name.expanded().local == tag_name
            } else {
                false
            }
        })
    }

    pub fn iter_children<'a>(&'a self, parent: &'a DomEntry) -> impl Iterator<Item = &'a DomEntry> {
        parent
            .children
            .iter()
            .map(|id| self.entries.get(*id).expect("Could not find child in DOM"))
    }

    pub fn add_node(&mut self, new_member: MemberKind) -> NodeId {
        self.entries.insert(DomEntry {
            myself: new_member,
            parent: None,
            children: Default::default(),
        })
    }

    fn node(&self, node_id: NodeId) -> &DomEntry {
        self.entries
            .get(node_id)
            .expect("Could not find expected member")
    }

    pub fn parent_of(&self, node_id: NodeId) -> Option<NodeId> {
        self.entries.get(node_id)?.parent
    }

    pub fn prepend_to(&mut self, parent: NodeId, child: NodeId) {
        let [parent_entry, child_entry] = self
            .entries
            .get_disjoint_mut([parent, child])
            .expect("Could not find parent and/or child in DOM");

        child_entry.parent = Some(parent);
        parent_entry.children.push_front(child);
    }

    pub fn append_to(&mut self, parent: NodeId, child: NodeId) {
        let [parent_entry, child_entry] = self
            .entries
            .get_disjoint_mut([parent, child])
            .expect("Could not find parent and/or child in DOM");

        child_entry.parent = Some(parent);
        parent_entry.children.push_back(child);
    }

    pub fn insert_before(&mut self, parent: NodeId, sibling: NodeId, child: NodeId) {
        let [parent_entry, child_entry] = self
            .entries
            .get_disjoint_mut([parent, child])
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
            .get_disjoint_mut([parent, child])
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
