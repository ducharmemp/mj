use ecow::EcoString;
use hashbrown::HashMap;
use html5ever::QualName;
use ordermap::OrderMap;
use stakker::{ret, Actor, Ret, CX};

use crate::parser::NodeId;

mod appending;
mod inserting;
mod merging;
mod relationships;

#[derive(Debug, Clone)]
pub enum MemberKind {
    Document,
    Element {
        name: QualName,
        attrs: HashMap<QualName, EcoString>,
    },
    Comment {
        content: EcoString,
    },
    Text {
        contents: EcoString,
    },
}

impl MemberKind {
    pub fn text_contents(&self) -> Option<EcoString> {
        if let Self::Text { contents } = self {
            Some(contents.clone())
        } else {
            None
        }
    }

    pub fn append_text_content(&mut self, new_suffix: &str) {
        match self {
            MemberKind::Text { contents } => contents.push_str(new_suffix),
            _ => unimplemented!(),
        }
    }

    pub fn is_text(&mut self) -> bool {
        match self {
            Self::Text { .. } => true,
            _ => false,
        }
    }
}

pub struct DomEntry {
    id: NodeId,
    root: Actor<DomEntry>,
    parent: Option<Actor<DomEntry>>,
    children: OrderMap<NodeId, Actor<DomEntry>>,
    myself: MemberKind,
}

impl DomEntry {
    pub fn empty_of_kind(
        cx: CX![],
        id: NodeId,
        root: Actor<DomEntry>,
        kind: MemberKind,
    ) -> Option<Self> {
        Some(Self {
            id,
            root,
            parent: None,
            children: OrderMap::new(),
            myself: kind,
        })
    }

    pub fn normalize(&mut self, cx: CX![]) {}

    fn is_text(&mut self, cx: CX![], callback: Ret<bool>) {
        ret!([callback], self.myself.is_text())
    }

    pub fn debug(&mut self, cx: CX![]) {
        match &self.myself {
            MemberKind::Document => {
                dbg!("Document Root");
            }
            MemberKind::Element { name, attrs } => {
                dbg!(name);
            }
            MemberKind::Comment { content } => {
                dbg!(content);
            }
            MemberKind::Text { contents } => {
                dbg!(contents);
            }
        };
    }
    pub fn id(&mut self, cx: CX![], callback: Ret<NodeId>) {
        ret!([callback], self.id)
    }

    pub fn text_content(&mut self, cx: CX![], callback: Ret<Option<EcoString>>) {
        // Todo: call recursively if we're not a text content element
        ret!([callback], self.myself.text_contents())
    }

    pub fn append_text(&mut self, cx: CX![], new_suffix: EcoString) {
        // Todo: Wipe children if we're not a text node
        if self.myself.is_text() {
            self.myself.append_text_content(&new_suffix)
        }
    }
}
