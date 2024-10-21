use std::{
    borrow::Cow,
    collections::VecDeque,
    io::BufReader,
    sync::atomic::{AtomicUsize, Ordering},
};

use hashbrown::HashMap;
use html5ever::{
    interface::{
        ElementFlags,
        NodeOrText::{self, AppendNode, AppendText},
        QuirksMode, TreeSink,
    },
    parse_document,
    tendril::{StrTendril, TendrilSink},
    Attribute, ExpandedName, QualName,
};
use stakker::PipedLink;

pub(crate) type NodeId = usize;
static NEXT_NODE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct ParserNodeElement {
    pub id: NodeId,
    pub name: Option<QualName>,
}

#[derive(Clone, Debug)]
pub struct ParserAttribute {
    name: QualName,
    value: String,
}

#[derive(Clone, Debug)]
pub enum ParserNodeOrText {
    Node(ParserNodeElement),
    Text(NodeId, String),
}

#[derive(Clone, Debug)]
pub enum ParseOperation {
    GetTemplateContents {
        target: NodeId,
        contents: NodeId,
    },

    CreateElement {
        node: NodeId,
        name: QualName,
        attrs: Vec<ParserAttribute>,
        current_line: u64,
    },

    CreateComment {
        text: String,
        node: NodeId,
    },
    AppendBeforeSibling {
        sibling: NodeId,
        node: ParserNodeOrText,
    },
    AppendBasedOnParentNode {
        element: NodeId,
        prev_element: NodeId,
        node: ParserNodeOrText,
    },
    Append {
        parent: NodeId,
        node: ParserNodeOrText,
    },

    AppendDoctypeToDocument {
        name: String,
        public_id: String,
        system_id: String,
    },

    AddAttrsIfMissing {
        target: NodeId,
        attrs: Vec<ParserAttribute>,
    },
    RemoveFromParent {
        target: NodeId,
    },
    MarkScriptAlreadyStarted {
        node: NodeId,
    },
    ReparentChildren {
        parent: NodeId,
        new_parent: NodeId,
    },

    AssociateWithForm {
        target: NodeId,
        form: NodeId,
        element: NodeId,
        prev_element: Option<NodeId>,
    },

    CreatePI {
        node: NodeId,
        target: String,
        data: String,
    },

    Pop {
        node: NodeId,
    },

    SetQuirksMode {
        mode: String,
    },

    FinishedParsing,
}

pub struct MjDomParser<'parser> {
    document_node: NodeId,
    entries: HashMap<NodeId, ParserNodeElement>,

    link: &'parser mut PipedLink<String, ParseOperation>,
}

impl<'parser> MjDomParser<'parser> {
    pub fn new(link: &'parser mut PipedLink<String, ParseOperation>) -> Self {
        let mut parser = Self {
            link,
            document_node: 0,
            entries: HashMap::new(),
        };
        let root_id = parser.add_root();
        parser.document_node = root_id;
        parser
    }

    fn add_root(&mut self) -> NodeId {
        let node_id = NEXT_NODE_ID.fetch_add(1, Ordering::SeqCst);
        self.entries.insert(
            node_id,
            ParserNodeElement {
                id: node_id,
                name: None,
            },
        );
        node_id
    }

    fn add_element(&mut self, name: QualName) -> NodeId {
        let node_id = NEXT_NODE_ID.fetch_add(1, Ordering::SeqCst);
        self.entries.insert(
            node_id,
            ParserNodeElement {
                id: node_id,
                name: Some(name),
            },
        );
        node_id
    }

    fn add_text(&mut self) -> NodeId {
        let node_id = NEXT_NODE_ID.fetch_add(1, Ordering::SeqCst);
        self.entries.insert(
            node_id,
            ParserNodeElement {
                id: node_id,
                name: None,
            },
        );
        node_id
    }

    pub fn add_comment(&mut self) -> NodeId {
        let node_id = NEXT_NODE_ID.fetch_add(1, Ordering::SeqCst);
        self.entries.insert(
            node_id,
            ParserNodeElement {
                id: node_id,
                name: None,
            },
        );
        node_id
    }

    fn node(&self, node_id: NodeId) -> &ParserNodeElement {
        self.entries
            .get(&node_id)
            .expect("Could not find expected member")
    }
}

impl<'parse_context, 'parser: 'parse_context> TreeSink for MjDomParser<'parser> {
    type Handle = ParserNodeElement;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self.link.send(ParseOperation::FinishedParsing);
        self
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        println!("Parse error: {}", msg);
    }

    fn get_document(&mut self) -> Self::Handle {
        Self::Handle {
            id: self.document_node,
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

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {
        self.node(target.id)
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
        let node_id = self.add_element(name.clone());

        let attrs = attributes
            .iter()
            .map(|attr| ParserAttribute {
                name: attr.name.clone(),
                value: attr.value.to_string(),
            })
            .collect();
        self.link.send(ParseOperation::CreateElement {
            node: node_id,
            name: name.clone(),
            attrs,
            current_line: 1,
        });

        Self::Handle {
            id: node_id,
            name: Some(name),
        }
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node_id = self.add_comment();
        self.link.send(ParseOperation::CreateComment {
            node: node_id,
            text: String::from(text),
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
        let node_id = self.add_text();
        self.link.send(ParseOperation::Append {
            parent: parent.id,
            node: match child {
                AppendNode(node) => ParserNodeOrText::Node(node),
                AppendText(content) => {
                    let content = String::from(content);
                    ParserNodeOrText::Text(node_id, content)
                }
            },
        });
    }

    fn append_before_sibling(&mut self, sibling: &Self::Handle, child: NodeOrText<Self::Handle>) {}

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        _prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
    }

    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        println!("Append doctype: {} {} {}", name, public_id, system_id);
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {}

    fn associate_with_form(
        &mut self,
        _target: &Self::Handle,
        _form: &Self::Handle,
        _nodes: (&Self::Handle, Option<&Self::Handle>),
    ) {
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {}

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {}

    fn mark_script_already_started(&mut self, node: &Self::Handle) {
        todo!()
    }

    fn set_current_line(&mut self, line_number: u64) {}
}
