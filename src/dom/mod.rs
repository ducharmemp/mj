use std::io::BufReader;

use html5ever::{parse_document, tendril::TendrilSink};
use ractor::{async_trait, cast, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use repr::{DomEntry, DomTree};
use taffy::prelude::*;
use tracing::{event, instrument, Level};

use parser::DomSink;

use crate::webview::MjWebViewMessage;

mod parser;
mod repr;

pub struct MjDom;

pub type MjDomArgs = ActorRef<MjWebViewMessage>;

pub enum MjDomMessage {
    ParseDocument(Box<String>),
    ParseFragment(Box<String>),
    IntoLayout(RpcReplyPort<(taffy::NodeId, TaffyTree<()>)>),
}

pub struct MjDomState {
    dom: DomTree,
    webview: ActorRef<MjWebViewMessage>,
}

impl MjDom {
    fn create_layout(dom: &DomTree) -> (taffy::NodeId, TaffyTree) {
        let mut taffy = TaffyTree::new();
        let container = taffy.new_leaf(Default::default()).unwrap();

        let body = dom.body();
        if body.is_none() {
            return (container, taffy);
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

        let container_id = descent(dom, body, &mut taffy);
        (container_id, taffy)
    }
}

#[async_trait]
impl Actor for MjDom {
    type Msg = MjDomMessage;
    type State = MjDomState;
    type Arguments = MjDomArgs;

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: MjDomArgs,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting dom with default empty document");
        Ok(MjDomState {
            dom: Default::default(),
            webview: args,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MjDomMessage::ParseDocument(content) => {
                event!(Level::INFO, "Parsing document into new tree structure");
                let document = parse_document(DomSink::new(), Default::default())
                    .from_utf8()
                    .read_from(&mut BufReader::new(content.as_bytes()))
                    .unwrap();
                state.dom = document;
                cast!(state.webview, MjWebViewMessage::DomUpdated)?;
            }
            MjDomMessage::ParseFragment(_) => todo!(),
            MjDomMessage::IntoLayout(reply) => reply.send(Self::create_layout(&state.dom))?,
        };
        Ok(())
    }
}
