use stakker::{call, lazy, ret_some_to, Actor, CX};

use crate::parser::NodeId;

use super::DomEntry;

impl DomEntry {
    pub fn append(&mut self, cx: CX![], other: Actor<DomEntry>) {
        let callback = ret_some_to!([cx], inner_append(other.clone()) as (NodeId));

        call!([other], id(callback));
    }

    fn inner_append(&mut self, cx: CX![], to_insert: Actor<DomEntry>, actor_id: NodeId) {
        call!([to_insert], set_parent(cx.this().clone().into()));
        self.children.insert(actor_id, to_insert);
    }
}
