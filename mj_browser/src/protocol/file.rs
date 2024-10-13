use std::{fs::File, io::Read};

use stakker::{ret, stop, Ret, CX};
use stakker_log::info;
use url::Url;

pub struct MjFileHandler;

impl MjFileHandler {
    pub fn init(cx: CX![]) -> Option<Self> {
        Some(Self {})
    }

    pub fn fetch(&mut self, cx: CX![], url: Url, ret: Ret<String>) {
        info!([cx], "Fetching {}", url);
        let url = url
            .to_file_path()
            .expect("Could not convert url to file path");
        let mut buf = String::new();
        File::open(url).unwrap().read_to_string(&mut buf).unwrap();
        ret!([ret], buf);
        stop!(cx);
    }
}
