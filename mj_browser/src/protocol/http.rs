use stakker::{ret, stop, Ret, CX};
use stakker_log::info;
use url::Url;

pub struct MjHttpHandler;

impl MjHttpHandler {
    pub fn init(cx: CX![]) -> Option<Self> {
        Some(Self {})
    }

    pub fn fetch(&mut self, cx: CX![], url: Url, ret: Ret<String>) {
        info!([cx], "Fetching {}", url);
        let buf = ureq::get(url.as_ref())
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        ret!([ret], buf);
        stop!(cx);
    }
}
