use std::{fs::File, io::Read};

use stakker::{ret, Ret, CX};
use tracing::{event, instrument, Level};
use url::Url;

pub struct MjProtocolHandler;

impl MjProtocolHandler {
    #[instrument(skip(cx))]
    pub fn init(cx: CX![]) -> Option<Self> {
        event!(Level::INFO, "Starting protocol handler");
        Some(Self {})
    }

    pub fn fetch(&mut self, cx: CX![], url: Url, ret: Ret<String>) {
        match url.scheme() {
            "file" => {
                let url = url
                    .to_file_path()
                    .expect("Could not convert url to file path");
                let mut buf = String::new();
                File::open(url).unwrap().read_to_string(&mut buf).unwrap();
                ret!([ret], buf)
            }
            _ => todo!(),
        };
    }
}
