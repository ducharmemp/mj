use crate::protocol::handler::MjProtocolHandler;
use mj_dom::{layout::LayoutTree, tree::DomTree, MjDomParser};
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

    pub fn compute_layout(&mut self, cx: CX![]) -> LayoutTree {
        self.dom_tree.ro(cx).into()
    }
}
