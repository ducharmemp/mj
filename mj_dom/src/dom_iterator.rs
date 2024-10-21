use std::collections::VecDeque;

use mj_utilities::actor_iterator::ActorIterator;
use stakker::{actor, call, fwd, ret, ret_do, ret_to, stop, Actor, Fwd, Ret, CX};

use crate::nodes::dom_entry::DomEntry;

pub struct ForwardDomIterator {
    current: Actor<DomEntry>,
}

impl ActorIterator<Actor<DomEntry>> for ForwardDomIterator {
    fn next(&mut self, cx: CX![], callback: Ret<Actor<DomEntry>>) {
        let stepped_to_child = ret_to!(
            [cx],
            stepped_to_child(self.current.clone(), callback) as (Option<Actor<DomEntry>>)
        );
        call!([self.current], first_child(stepped_to_child));
    }
}

impl ForwardDomIterator {
    pub fn init(cx: CX![], start: Actor<DomEntry>) -> Option<Self> {
        Some(Self { current: start })
    }

    fn stepped_to_child(
        &mut self,
        cx: CX![],
        from: Actor<DomEntry>,
        callback: Ret<Actor<DomEntry>>,
        child: Option<Option<Actor<DomEntry>>>,
    ) {
        let child = child.flatten();
        match child {
            Some(mut new_current) => {
                std::mem::swap(&mut self.current, &mut new_current);
                ret!([callback], new_current);
            }
            None => {
                let stepped_to_sibling = ret_to!(
                    [cx],
                    stepped_to_sibling(from.clone(), callback) as (Option<Actor<DomEntry>>)
                );
                call!([from], next_sibling(stepped_to_sibling));
            }
        }
    }

    fn stepped_to_sibling(
        &mut self,
        cx: CX![],
        from: Actor<DomEntry>,
        callback: Ret<Actor<DomEntry>>,
        sibling: Option<Option<Actor<DomEntry>>>,
    ) {
        let sibling = sibling.flatten();
        match sibling {
            Some(mut new_current) => {
                std::mem::swap(&mut self.current, &mut new_current);
                ret!([callback], new_current);
            }
            None => {
                let stepped_to_parent = ret_to!(
                    [cx],
                    stepped_to_parent(from.clone(), callback) as (Option<Actor<DomEntry>>)
                );
                call!([from], parent(stepped_to_parent));
            }
        }
    }

    fn stepped_to_parent(
        &mut self,
        cx: CX![],
        from: Actor<DomEntry>,
        callback: Ret<Actor<DomEntry>>,
        parent: Option<Option<Actor<DomEntry>>>,
    ) {
        let parent = parent.flatten();
        match parent {
            Some(parent) => {
                let stepped_to_sibling = ret_to!(
                    [cx],
                    stepped_to_sibling(parent.clone(), callback) as (Option<Actor<DomEntry>>)
                );
                call!([parent], next_sibling(stepped_to_sibling));
            }
            None => {
                // Root of tree
                ret!([callback], self.current.clone());
                stop!(cx)
            }
        }
    }
}

struct ChildrenIterator {
    current: Actor<DomEntry>,
}

impl ChildrenIterator {
    pub fn init(cx: CX![], start_entry: Actor<DomEntry>) -> Option<Self> {
        Some(Self {
            current: start_entry,
        })
    }

    pub fn next(&mut self, cx: CX![], callback: Ret<Actor<DomEntry>>) {
        let ret = ret_to!([cx], recv_entry() as (Option<Actor<DomEntry>>));
        call!([self.current], next_sibling(ret));
        ret!([callback], self.current.clone());
    }

    fn recv_entry(&mut self, cx: CX![], new_entry: Option<Option<Actor<DomEntry>>>) {
        if let Some(entry) = new_entry.flatten() {
            self.current = entry;
        }
        stop!(cx);
    }
}
