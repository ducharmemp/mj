use std::collections::{HashMap, VecDeque};

use html5ever::QualName;

use crate::tree::NodeId;

#[derive(Debug, Clone)]
pub enum MemberKind {
    Root,
    Element {
        name: QualName,
        attrs: HashMap<QualName, String>,
    },
    Comment {
        content: String,
    },
    Text {
        contents: String,
    },
}

impl MemberKind {
    pub fn text_contents(&self) -> Option<&String> {
        if let Self::Text { contents } = self {
            Some(contents)
        } else {
            None
        }
    }

    pub fn text_contents_mut(&mut self) -> Option<&mut String> {
        if let Self::Text { contents } = self {
            Some(contents)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomEntry {
    pub parent: Option<NodeId>,
    pub myself: MemberKind,
    pub children: VecDeque<NodeId>,
}

impl DomEntry {}
