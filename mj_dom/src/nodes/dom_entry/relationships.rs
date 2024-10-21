use stakker::{call, ret, Actor, Ret, CX};

use crate::parser::NodeId;

use super::DomEntry;

impl DomEntry {
    pub fn set_parent(&mut self, cx: CX![], other: Option<Actor<DomEntry>>) {
        self.parent = other;
    }

    pub fn parent(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.parent.clone())
    }

    pub fn previous_sibling(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        let parent = self.parent.clone();
        match parent {
            None => ret!([callback], None),
            Some(parent) => call!([parent], previous_sibling_to(self.id, callback)),
        };
    }

    fn previous_sibling_to(
        &mut self,
        cx: CX![],
        node_id: NodeId,
        callback: Ret<Option<Actor<DomEntry>>>,
    ) {
        let child_index = self.children.get_index_of(&node_id);
        let child = match child_index {
            None => None,
            Some(0) => None,
            Some(idx) => self.children.get_index(idx - 1).map(|(_k, v)| v).cloned(),
        };
        ret!([callback], child)
    }

    pub fn next_sibling(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        let parent = self.parent.clone();
        match parent {
            None => ret!([callback], None),
            Some(parent) => call!([parent], next_sibling_to(self.id, callback)),
        };
    }

    fn next_sibling_to(
        &mut self,
        cx: CX![],
        node_id: NodeId,
        callback: Ret<Option<Actor<DomEntry>>>,
    ) {
        let child_index = self.children.get_index_of(&node_id);
        match child_index {
            Some(idx) if idx < self.children.len() - 1 => {
                ret!(
                    [callback],
                    self.children.get_index(idx + 1).map(|(_k, v)| v).cloned()
                );
            }
            _ => {
                ret!([callback], None);
            }
        };
    }

    pub fn first_child(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.inner_first_child())
    }

    fn inner_first_child(&mut self) -> Option<Actor<DomEntry>> {
        self.children.first().map(|(_k, v)| v).cloned()
    }

    pub fn last_child(&mut self, cx: CX![], callback: Ret<Option<Actor<DomEntry>>>) {
        ret!([callback], self.inner_last_child())
    }

    fn inner_last_child(&mut self) -> Option<Actor<DomEntry>> {
        self.children.last().map(|(_k, v)| v).cloned()
    }
}
