use crate::protocol::handler::MjProtocolHandler;
use mj_dom::MjDom;
use stakker::{actor, call, ret_nop, ret_shutdown, ret_some_to, ActorOwn, CX};
use url::Url;

pub struct MjWebview {
    url: Url,
    dom: ActorOwn<MjDom>,
    protocol_handler: ActorOwn<MjProtocolHandler>,
}

impl MjWebview {
    pub fn init(cx: CX![], url: Url) -> Option<Self> {
        let dom = actor!(cx, MjDom::init(), ret_shutdown!(cx));
        let protocol_handler = actor!(cx, MjProtocolHandler::init(), ret_nop!());
        let fetch_ret = ret_some_to!([dom], parse_document() as (String));
        call!([protocol_handler], fetch(url.clone(), fetch_ret));

        Some(Self {
            dom,
            url,
            protocol_handler,
        })
    }

    pub fn compute_layout(&mut self, cx: CX![]) {}
}
