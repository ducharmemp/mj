use std::{fs::File, io::Read};

use stakker::{actor_in_slab, call, ret, ActorOwnSlab, Ret, CX};
use tracing::{event, instrument, Level};
use url::Url;

use super::{file::MjFileHandler, http::MjHttpHandler};

pub struct MjProtocolHandler {
    file_slab: ActorOwnSlab<MjFileHandler>,
    http_slab: ActorOwnSlab<MjHttpHandler>,
}

impl MjProtocolHandler {
    #[instrument(skip(cx))]
    pub fn init(cx: CX![]) -> Option<Self> {
        event!(Level::INFO, "Starting protocol handler");
        Some(Self {
            file_slab: ActorOwnSlab::new(),
            http_slab: ActorOwnSlab::new(),
        })
    }

    pub fn fetch(&mut self, cx: CX![], url: Url, ret: Ret<String>) {
        match url.scheme() {
            "file" => {
                let actor = actor_in_slab!(self.file_slab, cx, MjFileHandler::init());
                call!([actor], fetch(url, ret))
            }
            "http" | "https" => {
                let actor = actor_in_slab!(self.http_slab, cx, MjHttpHandler::init());
                call!([actor], fetch(url, ret))
            }
            _ => unimplemented!(),
        };
    }
}
