use crate::protocol::handler::MjProtocolHandler;
use mj_dom::{
    repr::{DomEntry, DomTree},
    MjDomParser,
};
use stakker::{actor, call, ret_nop, ret_some_to, ActorOwn, Share, CX};
use taffy::{prelude::length, Display, Size, Style, TaffyTree};
use tracing::{event, instrument, Level};
use url::Url;

pub struct MjWebview {
    url: Url,
    dom_parser: ActorOwn<MjDomParser>,
    dom_tree: Share<DomTree>,
    protocol_handler: ActorOwn<MjProtocolHandler>,
}

impl MjWebview {
    #[instrument(skip(cx))]
    pub fn init(cx: CX![], url: Url) -> Option<Self> {
        event!(Level::INFO, "Starting new webview");
        let dom_tree = Share::new(cx, DomTree::new());
        let dom_parser = actor!(cx, MjDomParser::init(dom_tree.clone()), ret_nop!());
        let protocol_handler = actor!(cx, MjProtocolHandler::init(), ret_nop!());
        let fetch_ret = ret_some_to!([dom_parser], parse_document() as (String));
        call!([protocol_handler], fetch(url.clone(), fetch_ret));

        Some(Self {
            dom_parser,
            url,
            protocol_handler,
            dom_tree,
        })
    }

    pub fn compute_layout(&mut self, cx: CX![]) -> (taffy::NodeId, TaffyTree) {
        let mut taffy = TaffyTree::new();
        let container = taffy.new_leaf(Default::default()).unwrap();

        let body = self.dom_tree.ro(&cx).body();
        if body.is_none() {
            dbg!("No body");
            return (container, taffy);
        }
        let body = body.unwrap();
        fn descent(dom: &DomTree, node: &DomEntry, builder: &mut TaffyTree) -> taffy::NodeId {
            let mut mapped = vec![];
            for child in dom.iter_children(node) {
                mapped.push(descent(dom, child, builder));
            }
            builder
                .new_with_children(
                    Style {
                        size: Size {
                            width: length(100.0),
                            height: length(100.0),
                        },
                        display: Display::Block,
                        ..Default::default()
                    },
                    &mapped,
                )
                .expect("Could not create new node")
        }

        let container_id = descent(&self.dom_tree.ro(&cx), body, &mut taffy);
        return (container_id, taffy);
    }
}
