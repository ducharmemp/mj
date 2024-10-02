use std::io::BufReader;

use html5ever::{parse_document, tendril::TendrilSink};
use repr::{DomEntry, DomTree};
use stakker::{ret, Ret, CX};
use taffy::prelude::*;
use tracing::{event, instrument, Level};

use parser::DomSink;

mod parser;
mod repr;

pub struct MjDom {
    dom: DomTree,
}

impl MjDom {
    #[instrument(skip(cx))]
    pub fn init(cx: CX![]) -> Option<Self> {
        event!(Level::INFO, "Starting dom with default empty document");
        Some(MjDom {
            dom: Default::default(),
        })
    }

    pub fn parse_document(&mut self, _cx: CX![], content: String) {
        dbg!(&content);
        let document = parse_document(DomSink::new(), Default::default())
            .from_utf8()
            .read_from(&mut BufReader::new(content.as_bytes()))
            .unwrap();
        self.dom = document;
    }

    pub fn into_layout(&mut self, _cx: CX![], ret: Ret<(taffy::NodeId, taffy::TaffyTree)>) {
        let mut taffy = TaffyTree::new();
        let container = taffy.new_leaf(Default::default()).unwrap();

        let body = self.dom.body();
        if body.is_none() {
            ret!([ret], (container, taffy));
            return;
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
                        display: Display::Flex,
                        ..Default::default()
                    },
                    &mapped,
                )
                .expect("Could not create new node")
        }

        let container_id = descent(&self.dom, body, &mut taffy);
        ret!([ret], (container, taffy));
    }
}
