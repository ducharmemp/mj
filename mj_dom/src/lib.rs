use std::io::BufReader;

use html5ever::{parse_document, tendril::TendrilSink};
use stakker::{ret, Ret, Share, CX};
use taffy::prelude::*;
use tracing::{event, instrument, Level};
use tree::DomTree;

pub mod document;
pub mod layout;
pub mod node;
pub mod tree;

pub struct MjDomParser {
    dom: Share<DomTree>,
}

impl MjDomParser {
    #[instrument(skip(cx, dom))]
    pub fn init(cx: CX![], dom: Share<DomTree>) -> Option<Self> {
        event!(Level::INFO, "Starting dom with default empty document");
        Some(MjDomParser { dom })
    }

    pub fn parse_document(&mut self, cx: CX![], content: String) {
        let document = parse_document(DomTree::new(), Default::default())
            .from_utf8()
            .read_from(&mut BufReader::new(content.as_bytes()))
            .unwrap();
        *self.dom.rw(cx) = document;
    }
}
