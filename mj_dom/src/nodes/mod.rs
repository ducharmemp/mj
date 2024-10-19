use std::collections::{HashMap, VecDeque};

use html5ever::QualName;
use stakker::{call, ret, ret_to, Actor, Ret, CX};

use crate::parser::NodeId;

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
    pub id: NodeId,
    pub root: Actor<DomEntry>,
    pub parent: Option<Actor<DomEntry>>,
    pub first_child: Option<Actor<DomEntry>>,
    pub last_child: Option<Actor<DomEntry>>,
    pub previous_sibling: Option<Actor<DomEntry>>,
    pub next_sibling: Option<Actor<DomEntry>>,
    pub myself: MemberKind,
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
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            myself: kind,
        })
    }

    pub fn append(&mut self, cx: CX![], other: Actor<DomEntry>) {
        let new_child = other.clone();
        call!([new_child], set_parent(cx.this().clone().into()));
        if self.first_child.is_none() {
            self.first_child = Some(new_child.clone());
            self.last_child = Some(new_child);
            return;
        }

        let sibling = other.clone().into();
        let last_child = self
            .last_child
            .clone()
            .expect("Last child cannot be none if the first child is populated");

        call!([last_child], set_next_sibling(sibling));
        call!([other], set_previous_sibling(self.last_child.clone()));
        self.last_child = Some(other);
    }

    pub fn insert_before(&mut self, cx: CX![], other: Actor<DomEntry>, before: Actor<DomEntry>) {
        let ret = ret_to!(
            [cx],
            insert_resolve_relationships(other.clone(), before.clone())
                as (Option<Actor<DomEntry>>)
        );
        call!([other], set_parent(cx.this().clone().into()));
        call!([before], previous_sibling(ret))
    }

    fn insert_resolve_relationships(
        &mut self,
        cx: CX![],
        other: Actor<DomEntry>,
        before: Actor<DomEntry>,
        previous: Option<Option<Actor<DomEntry>>>,
    ) {
        if let Some(previous) = previous.flatten() {
            let new_sibling = previous.clone();
            let to_insert = other.clone();
            call!([new_sibling], set_next_sibling(other.clone().into()));
            call!([to_insert], set_previous_sibling(previous.into()))
        } else {
            // If the node didn't have a sibling to the left, it was the first one
            self.first_child = other.clone().into();
        }
        let to_insert = other.clone();
        call!([to_insert], set_next_sibling(before.clone().into()));
        call!([before], set_previous_sibling(other.into()));
    }

    pub fn set_parent(&mut self, cx: CX![], other: Option<Actor<DomEntry>>) {
        self.parent = other;
    }

    pub fn parent(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.parent.clone())
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
        ret!([callback], self.first_child.clone());
    }

    pub fn last_child(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.last_child.clone());
    }

    pub fn id(&mut self, cx: CX![], callback: Ret<NodeId>) {
        ret!([callback], self.id);
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
}
