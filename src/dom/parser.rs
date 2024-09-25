use std::borrow::Cow;

use html5ever::{
    interface::{
        ElementFlags,
        NodeOrText::{self, AppendNode, AppendText},
        QuirksMode, TreeSink,
    },
    tendril::StrTendril,
    Attribute, ExpandedName, QualName,
};

use super::repr::{DomTree, MemberKind, NodeId};

#[derive(Clone)]
pub struct DomElementHandle {
    pub id: NodeId,
    pub name: Option<QualName>,
}

pub(super) struct DomSink {
    line_no: u64,
    dom_layout: DomTree,
    root_id: NodeId,
}

impl DomSink {
    pub fn new() -> Self {
        let (dom_layout, root_id) = DomTree::new();
        Self {
            line_no: 1,
            dom_layout,
            root_id,
        }
    }
}

impl TreeSink for DomSink {
    type Handle = DomElementHandle;
    type Output = DomTree;

    fn finish(self) -> Self::Output {
        self.dom_layout
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
        let node_id = self
            .dom_layout
            .add_node(MemberKind::Element { name: name.clone() });
        Self::Handle {
            id: node_id,
            name: Some(name),
        }
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node_id = self.dom_layout.add_node(MemberKind::Comment {
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
            AppendNode(node) => self.dom_layout.append_to(parent.id, node.id),
            AppendText(text) => {
                let node_id = self.dom_layout.add_node(MemberKind::Text {
                    contents: text.to_string(),
                });
                self.dom_layout.append_to(parent.id, node_id)
            }
        };
    }

    fn append_before_sibling(&mut self, sibling: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let parent = self
            .dom_layout
            .parent_of(sibling.id)
            .expect("Element has no parent");
        match child {
            AppendNode(node) => self.dom_layout.insert_before(parent, sibling.id, node.id),
            AppendText(text) => {
                let node_id = self.dom_layout.add_node(MemberKind::Text {
                    contents: text.to_string(),
                });
                self.dom_layout.append_to(parent, node_id)
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

    fn set_current_line(&mut self, line_number: u64) {
        self.line_no = line_number;
    }
}
