use std::collections::{HashMap, VecDeque};

use html5ever::QualName;
use stakker::{call, ret, Actor, Ret, CX};

pub mod document;

#[derive(Debug, Clone)]
pub enum MemberKind {
    Document,
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

#[derive(Clone)]
pub struct DomEntry {
    pub parent: Option<Actor<DomEntry>>,
    pub previous_sibling: Option<Actor<DomEntry>>,
    pub next_sibling: Option<Actor<DomEntry>>,
    pub myself: MemberKind,
    pub children: VecDeque<Actor<DomEntry>>,
}

impl DomEntry {
    pub fn empty_of_kind(cx: CX![], kind: MemberKind) -> Option<Self> {
        Some(Self {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            myself: kind,
            children: Default::default(),
        })
    }

    pub fn append(&mut self, cx: CX![], other: Actor<DomEntry>) {
        if !self.children.is_empty() {
            let new_child = other.clone();
            let prev_sibling = self.children.get(self.children.len()).cloned();
            call!([new_child], set_previous_sibling(prev_sibling));
        }
        self.children.push_back(other);
    }

    pub fn set_previous_sibling(&mut self, cx: CX![], other: Option<Actor<DomEntry>>) {
        self.previous_sibling = other;
    }
    pub fn previous_sibling(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.previous_sibling.clone());
    }

    pub fn set_next_sibling(&mut self, cx: CX![], other: Option<Actor<DomEntry>>) {
        self.next_sibling = other;
    }
    pub fn next_sibling(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.next_sibling.clone());
    }

    pub fn first_child(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.children.get(0).cloned());
    }

    pub fn last_child(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!(
            [callback],
            self.children.get(self.children.len() - 1).cloned()
        );
    }
}
