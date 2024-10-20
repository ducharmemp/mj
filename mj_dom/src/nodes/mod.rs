use std::collections::{HashMap, VecDeque};

use ecow::EcoString;
use html5ever::QualName;
use stakker::{call, ret, ret_do, ret_some_do, ret_to, stop, Actor, Ret, CX};

use crate::parser::NodeId;

pub mod document;

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
        if let Self::Text { ref mut contents } = self {
            dbg!(&contents);
            dbg!(&new_suffix);
            contents.push_str(new_suffix)
        } else {
            unimplemented!()
        }
    }

    pub fn is_text(&mut self) -> bool {
        match self {
            Self::Text { .. } => true,
            _ => false,
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

    fn is_text(&mut self, cx: CX![], callback: Ret<bool>) {
        ret!([callback], self.myself.is_text())
    }
}

impl DomEntry {
    pub fn append(&mut self, cx: CX![], other: Actor<DomEntry>) {
        let new_child = other.clone();
        call!([new_child], set_parent(cx.this().clone().into()));
        if self.first_child.is_none() {
            self.first_child = Some(new_child.clone());
            self.last_child = Some(new_child);
            return;
        }

        let sibling = other.clone().into();
        let current_last_child = self
            .last_child
            .clone()
            .expect("Last child cannot be none if the first child is populated");

        call!([current_last_child], set_next_sibling(sibling));
        call!(
            [other],
            set_previous_sibling(current_last_child.clone().into())
        );
        self.last_child = Some(other.clone());
        call!([current_last_child], maybe_merge(other))
    }

    pub fn insert_before(&mut self, cx: CX![], other: Actor<DomEntry>, next: Actor<DomEntry>) {
        let ret = ret_to!(
            [cx],
            insert_resolve_relationships(other.clone(), next.clone()) as (Option<Actor<DomEntry>>)
        );
        call!([other], set_parent(cx.this().clone().into()));
        call!([next], previous_sibling(ret))
    }

    fn insert_resolve_relationships(
        &mut self,
        _cx: CX![],
        between: Actor<DomEntry>,
        next: Actor<DomEntry>,
        previous: Option<Option<Actor<DomEntry>>>,
    ) {
        if let Some(previous) = previous.flatten() {
            let new_sibling = previous.clone();
            let to_insert = between.clone();
            call!([new_sibling], set_next_sibling(between.clone().into()));
            call!([to_insert], set_previous_sibling(previous.clone().into()));
            call!([previous], maybe_merge(to_insert));
        } else {
            // If the node didn't have a sibling to the left, it was the first one
            self.first_child = between.clone().into();
        }
        let to_insert = between.clone();
        call!([to_insert], set_next_sibling(next.clone().into()));
        call!([next], set_previous_sibling(between.into()));
    }

    fn maybe_merge(&mut self, cx: CX![], other: Actor<DomEntry>) {
        if !self.myself.is_text() {
            return;
        }

        let this = cx.this().clone();
        let to_merge = other.clone();
        let is_text_cb = ret_some_do!(move |other_is_text: bool| {
            if !other_is_text {
                return;
            }
            call!([this], merge(to_merge))
        });

        call!([other], is_text(is_text_cb));
    }

    fn merge(&mut self, cx: CX![], other: Actor<DomEntry>) {
        let this = cx.this().clone();
        let bunk = other.clone();
        let text_cb = ret_some_do!(move |other_text_content: EcoString| {
            call!([this], append_text_content(other_text_content));
            call!([bunk], remove_self());
        });
        call!([other], text_content(text_cb));
    }

    pub fn remove_child(&mut self, cx: CX![], child: Actor<DomEntry>) {
        // TODO: Need to verify that yes, the child is actually one of self's
        call!([child], remove_self())
    }

    fn remove_self(&mut self, cx: CX![]) {
        println!("Removing self");
        self.debug(cx);
        let parent = self
            .parent
            .clone()
            .expect("Trying to remove a parentless entry");
        if let Some(previous_sibling) = &self.previous_sibling {
            call!(
                [previous_sibling],
                set_next_sibling(self.next_sibling.clone())
            );
        }

        if let Some(next_sibling) = &self.next_sibling {
            call!(
                [next_sibling],
                set_previous_sibling(self.previous_sibling.clone())
            );
        }

        // stop!(cx);
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

impl DomEntry {
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

    fn set_first_child(&mut self, cx: CX![], child: Option<Actor<DomEntry>>) {
        self.first_child = child;
    }

    pub fn last_child(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.last_child.clone());
    }

    fn set_last_child(&mut self, cx: CX![], child: Option<Actor<DomEntry>>) {
        self.first_child = child;
    }

    pub fn id(&mut self, cx: CX![], callback: Ret<NodeId>) {
        ret!([callback], self.id);
    }

    pub fn text_content(&mut self, cx: CX![], callback: Ret<EcoString>) {
        // Todo: call recursively if we're not a text content element
        if self.myself.is_text() {
            ret!([callback], self.myself.text_contents().unwrap())
        }
    }

    pub fn append_text_content(&mut self, cx: CX![], new_suffix: EcoString) {
        // Todo: Wipe children if we're not a text node
        if self.myself.is_text() {
            self.myself.append_text_content(&new_suffix)
        }
    }
}
