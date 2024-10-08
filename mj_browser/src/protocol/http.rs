use std::{fs::File, io::Read};

use stakker::{ret, stop, Ret, CX};
use tracing::{event, instrument, Level};
use url::Url;

pub struct MjHttpHandler;

impl MjHttpHandler {
    #[instrument(skip(cx))]
    pub fn init(cx: CX![]) -> Option<Self> {
        event!(Level::INFO, "Starting protocol handler");
        Some(Self {})
    }

    pub fn fetch(&mut self, cx: CX![], url: Url, ret: Ret<String>) {
        let buf = ureq::get(&url.to_string())
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        ret!([ret], buf);
        stop!(cx);
    }
}
