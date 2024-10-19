use mj_dom::{dom_iterator::ForwardDomIterator, nodes::DomEntry, MjDom};
use mj_utilities::actor_iterator::ActorIterator;
use stakker::{call, ret_do, ret_nop, ret_some_do, ret_some_to, Actor, ActorOwn, CX};

pub struct MjLayout {
    width: u32,
    height: u32,
    dom: Actor<MjDom>,
}

impl MjLayout {
    pub fn init(cx: CX![], dom: Actor<MjDom>) -> Option<Self> {
        Some(Self {
            width: 0,
            height: 0,
            dom,
        })
    }

    pub fn set_size(&mut self, cx: CX![], width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn reflow(&mut self, cx: CX![]) {
        let callback = ret_some_to!(
            [cx],
            rebuild_layout_tree(None) as (ActorOwn<ForwardDomIterator>)
        );
        call!([self.dom], iter(callback));
    }
}

impl MjLayout {
    fn rebuild_layout_tree(
        &mut self,
        cx: CX![],
        node: Option<Actor<DomEntry>>,
        iterator: ActorOwn<ForwardDomIterator>,
    ) {
        let this = cx.this().clone();
        let inner_iter = iterator.owned();
        let callback = ret_some_do!(move |node: Actor<DomEntry>| {
            let d = node.clone();
            call!([d], debug());
            call!([this], rebuild_layout_tree(Some(node), inner_iter))
        });
        call!([iterator], next(callback));
    }
}
