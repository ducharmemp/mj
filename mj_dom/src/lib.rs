use std::{borrow::Cow, collections::HashMap, hash::Hash, io::BufReader};

use dom_iterator::ForwardDomIterator;
use html5ever::{
    interface::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
    parse_document,
    tendril::{StrTendril, TendrilSink},
    Attribute, ExpandedName, QualName,
};
use mj_utilities::{actor_in_map, actor_new_in_map, actor_own_map::ActorOwnMap};
use nodes::{DomEntry, MemberKind};
use parser::{MjDomParser, NodeId, ParseOperation};
use stakker::{
    actor, actor_in_slab, call, fwd_to, ret, ret_do, ret_nop, Actor, ActorOwn, ActorOwnSlab, Cx,
    PipedLink, PipedThread, Ret, Share, CX,
};

// pub mod layout;
pub mod dom_iterator;
pub mod nodes;
pub mod parser;

pub struct MjDom {
    document: Option<Actor<DomEntry>>,
    nodes: ActorOwnMap<NodeId, DomEntry>,
    parser: PipedThread<String, ParseOperation>,
}

impl MjDom {
    pub fn init(cx: CX![]) -> Option<Self> {
        let dom = Self {
            document: None,
            nodes: ActorOwnMap::new(),
            parser: PipedThread::spawn(
                fwd_to!([cx], recv() as (ParseOperation)),
                fwd_to!([cx], parser_terminated() as (Option<String>)),
                cx,
                move |link| {
                    while let Some(message) = link.recv() {
                        let parser = MjDomParser::new(link);
                        parse_document(parser, Default::default())
                            .from_utf8()
                            .read_from(&mut BufReader::new(message.as_bytes()))
                            .unwrap();
                    }
                },
            ),
        };
        Some(dom)
    }

    pub fn parse_document(&mut self, cx: CX![], content: String) {
        let document = actor_new_in_map!(self.nodes, cx, 0);
        let root = document.clone();
        let initializer = document.clone();
        call!(
            [initializer],
            DomEntry::empty_of_kind(0, root, MemberKind::Document)
        );
        self.document = document.into();
        self.parser.send(content);
    }

    pub fn iter(&mut self, cx: CX![], callback: Ret<ActorOwn<ForwardDomIterator>>) {
        ret!(
            [callback],
            actor!(
                cx,
                ForwardDomIterator::init(self.document.clone().expect("No document")),
                ret_nop!()
            ) as (ActorOwn<ForwardDomIterator>)
        )
    }

    fn recv(&mut self, cx: CX![], message: ParseOperation) {
        dbg!(self.nodes.len());
        match message {
            ParseOperation::GetTemplateContents { target, contents } => todo!(),
            ParseOperation::CreateElement {
                node,
                name,
                attrs,
                current_line,
            } => {
                actor_in_map!(
                    self.nodes,
                    cx,
                    node,
                    DomEntry::empty_of_kind(
                        node,
                        self.document.clone().expect("Document must be present"),
                        MemberKind::Element {
                            name,
                            attrs: HashMap::new()
                        }
                    )
                );
            }
            ParseOperation::CreateComment { text, node } => {
                actor_in_map!(
                    self.nodes,
                    cx,
                    node,
                    DomEntry::empty_of_kind(
                        node,
                        self.document.clone().expect("Document must be present"),
                        MemberKind::Comment { content: text }
                    )
                );
            }
            ParseOperation::AppendBeforeSibling { sibling, node } => {
                match node {
                    parser::ParserNodeOrText::Node(node) => {
                        let [sibling_actor, actor] = self.nodes.get_many_mut([&sibling, &node.id]);
                        let sibling_actor =
                            sibling_actor.expect("Could not find sibling element in DOM");
                        let actor = actor.expect("Could not find element in DOM");
                        let parent_resolver = sibling_actor.clone();
                        let sibling_actor = sibling_actor.clone();
                        let actor = actor.clone();
                        call!(
                            [parent_resolver],
                            parent(ret_do!(move |parent: Option<Option<Actor<DomEntry>>>| {
                                let parent = parent
                                    .flatten()
                                    .expect("Could not get parent of sibling node");
                                call!([parent], insert_before(actor, sibling_actor))
                            }))
                        );
                    }
                    parser::ParserNodeOrText::Text(node_id, text) => {
                        let actor = {
                            actor_in_map!(
                                self.nodes,
                                cx,
                                node_id,
                                DomEntry::empty_of_kind(
                                    node_id,
                                    self.document.clone().expect("Document must be present"),
                                    MemberKind::Text { contents: text }
                                )
                            )
                        };
                        let sibling_actor = self
                            .nodes
                            .get(&sibling)
                            .expect("Could not find parent element in DOM");
                        call!([sibling_actor], append(actor))
                    }
                };
            }
            ParseOperation::AppendBasedOnParentNode {
                element,
                prev_element,
                node,
            } => todo!(),
            ParseOperation::Append { parent, node } => {
                match node {
                    parser::ParserNodeOrText::Node(node) => {
                        let [parent_actor, actor] = self.nodes.get_many_mut([&parent, &node.id]);
                        let parent_actor =
                            parent_actor.expect("Could not find parent element in DOM");
                        let actor = actor.expect("Could not find element in DOM");
                        let actor = actor.clone();
                        call!([parent_actor], append(actor));
                    }
                    parser::ParserNodeOrText::Text(node_id, text) => {
                        let actor = {
                            actor_in_map!(
                                self.nodes,
                                cx,
                                node_id,
                                DomEntry::empty_of_kind(
                                    node_id,
                                    self.document.clone().expect("Document must be present"),
                                    MemberKind::Text { contents: text }
                                )
                            )
                        };
                        let parent_actor = self
                            .nodes
                            .get(&parent)
                            .expect("Could not find parent element in DOM");
                        call!([parent_actor], append(actor))
                    }
                };
            }
            ParseOperation::AppendDoctypeToDocument {
                name,
                public_id,
                system_id,
            } => todo!(),
            ParseOperation::AddAttrsIfMissing { target, attrs } => todo!(),
            ParseOperation::RemoveFromParent { target } => todo!(),
            ParseOperation::MarkScriptAlreadyStarted { node } => todo!(),
            ParseOperation::ReparentChildren { parent, new_parent } => todo!(),
            ParseOperation::AssociateWithForm {
                target,
                form,
                element,
                prev_element,
            } => todo!(),
            ParseOperation::CreatePI { node, target, data } => todo!(),
            ParseOperation::Pop { node } => todo!(),
            ParseOperation::SetQuirksMode { mode } => todo!(),
        }
    }

    fn parser_terminated(&mut self, cx: CX![], panic: Option<String>) {
        if let Some(msg) = panic {
            panic!("Unexpected thread failure: {}", msg);
        }
        cx.stop();
    }
}
