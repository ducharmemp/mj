use std::{borrow::Cow, collections::VecDeque};

use html5ever::{
    interface::{
        ElementFlags,
        NodeOrText::{self, AppendNode, AppendText},
        QuirksMode, TreeSink,
    },
    tendril::StrTendril,
    Attribute, ExpandedName, QualName,
};
use slotmap::{new_key_type, SlotMap};

new_key_type! { pub struct NodeId; }

#[derive(Clone)]
pub struct DomElementHandle {
    pub id: NodeId,
    pub name: Option<QualName>,
}

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
    pub fn new() -> Self {
        let mut t = Self::default();
        let root_id = t.add_node(MemberKind::Root);
        t.root_id = root_id;
        t
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

impl TreeSink for DomTree {
    type Handle = DomElementHandle;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        println!("Parse error: {}", msg);
    }

    fn get_document(&mut self) -> Self::Handle {
        Self::Handle {
            id: self.root_id,
            name: None,
        }
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        todo!()
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        println!("Set quirks mode to {:?}", mode);
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn elem_name<'a>(&self, target: &'a Self::Handle) -> ExpandedName<'a> {
        target
            .name
            .as_ref()
            .expect("Could not get name of node")
            .expanded()
    }

    fn create_element(
        &mut self,
        name: QualName,
        _attributes: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        let node_id = self.add_node(MemberKind::Element { name: name.clone() });
        Self::Handle {
            id: node_id,
            name: Some(name),
        }
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node_id = self.add_node(MemberKind::Comment {
            content: text.to_string(),
        });
        Self::Handle {
            id: node_id,
            name: None,
        }
    }

    #[allow(unused_variables)]
    fn create_pi(&mut self, target: StrTendril, value: StrTendril) -> Self::Handle {
        todo!()
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            AppendNode(node) => self.append_to(parent.id, node.id),
            AppendText(text) => {
                let node_id = self.add_node(MemberKind::Text {
                    contents: text.to_string(),
                });
                self.append_to(parent.id, node_id)
            }
        };
    }

    fn append_before_sibling(&mut self, sibling: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let parent = self.parent_of(sibling.id).expect("Element has no parent");
        match child {
            AppendNode(node) => self.insert_before(parent, sibling.id, node.id),
            AppendText(text) => {
                let node_id = self.add_node(MemberKind::Text {
                    contents: text.to_string(),
                });
                self.append_to(parent, node_id)
            }
        };
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        _prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        self.append_before_sibling(element, child);
    }

    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        println!("Append doctype: {} {} {}", name, public_id, system_id);
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {
        for attr in attrs.into_iter() {
            println!("    {:?} = {}", attr.name, attr.value);
        }
    }

    fn associate_with_form(
        &mut self,
        _target: &Self::Handle,
        _form: &Self::Handle,
        _nodes: (&Self::Handle, Option<&Self::Handle>),
    ) {
        // No form owner support.
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        todo!()
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        todo!()
    }

    fn mark_script_already_started(&mut self, node: &Self::Handle) {
        todo!()
    }

    fn set_current_line(&mut self, line_number: u64) {}
}
