use stakker::{call, ret_some_do, ret_some_to, Actor, CX};

use crate::parser::NodeId;

use super::DomEntry;

impl DomEntry {
    pub fn insert_before(&mut self, cx: CX![], other: Actor<DomEntry>, next: Actor<DomEntry>) {
        let to_insert = other.clone();
        let this = cx.this().clone();
        let sibling_callback = ret_some_do!(move |sibling_node_id| {
            let callback = ret_some_to!(
                [this],
                inner_insert_before(to_insert.clone(), sibling_node_id) as (NodeId)
            );
            call!([to_insert], id(callback));
        });

        call!([next], id(sibling_callback));
    }

    fn inner_insert_before(
        &mut self,
        cx: CX![],
        actor: Actor<DomEntry>,
        next_id: NodeId,
        actor_id: NodeId,
    ) {
        let index_of = self
            .children
            .get_index_of(&next_id)
            .expect("Could not get index of sibling");
        self.children.insert_before(actor_id, index_of, actor);
    }
}
