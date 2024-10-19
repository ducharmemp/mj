use crate::protocol::handler::MjProtocolHandler;
use mj_dom::MjDom;
use mj_layout::MjLayout;
use stakker::{actor, call, ret_nop, ret_shutdown, ret_some_to, ActorOwn, CX};
use url::Url;

pub struct MjWebview {
    url: Url,
    dom: ActorOwn<MjDom>,
    layout: ActorOwn<MjLayout>,
    protocol_handler: ActorOwn<MjProtocolHandler>,
}

impl MjWebview {
    pub fn init(cx: CX![], url: Url) -> Option<Self> {
        let dom = actor!(cx, MjDom::init(), ret_shutdown!(cx));
        let layout = actor!(cx, MjLayout::init(dom.clone()), ret_shutdown!(cx));
        let protocol_handler = actor!(cx, MjProtocolHandler::init(), ret_nop!());
        let fetch_ret = ret_some_to!([dom], parse_document() as (String));
        call!([protocol_handler], fetch(url.clone(), fetch_ret));

        Some(Self {
            dom,
            layout,
            url,
            protocol_handler,
        })
    }

    pub fn set_content_area(&mut self, cx: CX![], width: u32, height: u32) {
        call!([self.layout], set_size(width, height));
    }

    pub fn composite(&mut self, cx: CX![]) {
        call!([self.layout], reflow())
    }
}
