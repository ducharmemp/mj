use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

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

use crate::node::{DomEntry, MemberKind};

new_key_type! { pub struct NodeId; }

#[derive(Clone)]
pub struct DomElementHandle {
    pub id: NodeId,
    pub name: Option<QualName>,
}

#[derive(Default, Debug)]
pub struct DomTree {
    root_id: NodeId,
    entries: SlotMap<NodeId, DomEntry>,
}

impl DomTree {
    pub fn new() -> Self {
        let mut t = Self::default();
        let root_id = t.add_root();
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

    pub fn add_root(&mut self) -> NodeId {
        self.entries.insert(DomEntry {
            myself: MemberKind::Root,
            parent: None,
            children: Default::default(),
            attrs: Default::default(),
        })
    }

    pub fn add_element(&mut self, name: QualName, attrs: HashMap<QualName, String>) -> NodeId {
        self.entries.insert(DomEntry {
            myself: MemberKind::Element { name: name },
            parent: None,
            children: Default::default(),
            attrs: Default::default(),
        })
    }

    pub fn add_text(&mut self, contents: String) -> NodeId {
        self.entries.insert(DomEntry {
            myself: MemberKind::Text { contents: contents },
            parent: None,
            children: Default::default(),
            attrs: Default::default(),
        })
    }

    pub fn add_comment(&mut self, content: String) -> NodeId {
        self.entries.insert(DomEntry {
            myself: MemberKind::Comment { content: content },
            parent: None,
            children: Default::default(),
            attrs: Default::default(),
        })
    }

    fn node(&self, node_id: NodeId) -> &DomEntry {
        self.entries
            .get(node_id)
            .expect("Could not find expected member")
    }

    fn node_mut(&mut self, node_id: NodeId) -> &mut DomEntry {
        self.entries
            .get_mut(node_id)
            .expect("Could not find expected member")
    }

    pub fn parent_of(&self, node_id: NodeId) -> Option<NodeId> {
        self.entries.get(node_id)?.parent
    }

    pub fn previous_sibling_of(&self, node_id: NodeId) -> Option<NodeId> {
        let parent = self.parent_of(node_id)?;
        let parent_entry = self.node(parent);
        let self_position = parent_entry
            .children
            .iter()
            .position(|id| *id == node_id)
            .expect("Could not find this node in the parent");
        if (self_position == 0) {
            return None;
        }
        parent_entry.children.get(self_position - 1).copied()
    }

    pub fn next_sibling_of(&self, node_id: NodeId) -> Option<NodeId> {
        let parent = self.parent_of(node_id)?;
        let parent_entry = self.node(parent);
        let self_position = parent_entry
            .children
            .iter()
            .position(|id| *id == node_id)
            .expect("Could not find this node in the parent");
        if (self_position == parent_entry.children.len() - 1) {
            return None;
        }
        parent_entry.children.get(self_position - 1).copied()
    }

    pub fn prepend_to(&mut self, parent: NodeId, child: NodeId) {
        {
            let [parent_entry, child_entry] = self
                .entries
                .get_disjoint_mut([parent, child])
                .expect("Could not find parent and/or child in DOM");

            child_entry.parent = Some(parent);
            parent_entry.children.push_front(child);
        }
        if let Some(sibling) = self.next_sibling_of(child) {
            self.maybe_merge_text(child, sibling);
        }
    }

    pub fn append_to(&mut self, parent: NodeId, child: NodeId) {
        {
            let [parent_entry, child_entry] = self
                .entries
                .get_disjoint_mut([parent, child])
                .expect("Could not find parent and/or child in DOM");

            child_entry.parent = Some(parent);
            parent_entry.children.push_back(child);
        }
        if let Some(sibling) = self.previous_sibling_of(child) {
            self.maybe_merge_text(sibling, child);
        }
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
        self.maybe_merge_text(child, sibling);
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
        self.maybe_merge_text(sibling, child);
    }

    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        let [parent_entry, child_entry] = self
            .entries
            .get_disjoint_mut([parent, child])
            .expect("Could not find parent and/or child in DOM");
        let child_position = parent_entry
            .children
            .iter()
            .position(|id| *id == child)
            .expect("Element is not a child of this parent element");
        child_entry.parent = None;
        parent_entry.children.remove(child_position);
    }

    fn maybe_merge_text(&mut self, left: NodeId, right: NodeId) {
        fn inner_concat(left_entry: &mut DomEntry, right_entry: &mut DomEntry) -> Option<()> {
            let left_text = left_entry.myself.text_contents_mut()?;
            let right_text = right_entry.myself.text_contents()?;
            left_text.push_str(right_text);
            Some(())
        }
        let [left_entry, right_entry] = self
            .entries
            .get_disjoint_mut([left, right])
            .expect("Could not find parent and/or child in DOM");
        if inner_concat(left_entry, right_entry).is_some() {
            self.remove_child(self.parent_of(right).expect("Node has no parent"), right)
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
        attributes: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        let node_id = self.add_element(
            name.clone(),
            attributes
                .iter()
                .map(|attr| (attr.name.clone(), attr.value.to_string()))
                .collect(),
        );
        Self::Handle {
            id: node_id,
            name: Some(name),
        }
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node_id = self.add_comment(text.to_string());
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
                let node_id = self.add_text(text.to_string());
                self.append_to(parent.id, node_id)
            }
        };
    }

    fn append_before_sibling(&mut self, sibling: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let parent = self.parent_of(sibling.id).expect("Element has no parent");
        match child {
            AppendNode(node) => self.insert_before(parent, sibling.id, node.id),
            AppendText(text) => {
                let node_id = self.add_text(text.to_string());
                self.insert_before(parent, sibling.id, node_id)
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
        let node = self.node_mut(target.id);
        node.attrs.extend(
            attrs
                .iter()
                .map(|attr| (attr.name.clone(), attr.value.to_string())),
        )
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
        let parent = self
            .parent_of(target.id)
            .expect("Target has no parent element");
        self.remove_child(parent, target.id)
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        let node = self.node_mut(node.id);
        let children = node.children.clone();
        node.children = VecDeque::new();
        for child in children {
            self.append_to(new_parent.id, child);
        }
    }

    fn mark_script_already_started(&mut self, node: &Self::Handle) {
        todo!()
    }

    fn set_current_line(&mut self, line_number: u64) {}
}
