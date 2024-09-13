use std::io::BufReader;

use html5ever::{parse_document, tendril::TendrilSink};
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

use parser::DomSink;

mod parser;
mod repr;

pub enum MjDomMessage {
    Parse(Box<String>),
}

pub struct MjDom;

#[async_trait]
impl Actor for MjDom {
    type Msg = MjDomMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MjDomMessage::Parse(content) => {
                parse_document(DomSink::default(), Default::default())
                    .from_utf8()
                    .read_from(&mut BufReader::new(content.as_bytes()))
                    .unwrap();
            }
        }
        Ok(())
    }
}
