use std::error::Error;

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use tracing::{event, instrument, Level};
use url::Url;

use super::{file::MjFileProtocolHandler, http::MjHttpProtocolHandler, MjProtocolMessage};

pub enum MjProtocolHandlerMessage {
    Fetch(Url, RpcReplyPort<ActorRef<MjProtocolMessage>>),
}

pub struct MjProtocolHandler;

pub struct MjProtocolHandlerState {}

impl MjProtocolHandler {
    async fn dispatch(url: Url) -> Result<ActorRef<MjProtocolMessage>, Box<dyn Error>> {
        let (actor, _) = match url.scheme() {
            "file" => {
                Actor::spawn(
                    None,
                    MjFileProtocolHandler,
                    url.to_file_path()
                        .expect("Could not convert url to file path"),
                )
                .await?
            }
            _ => Actor::spawn(None, MjHttpProtocolHandler, url).await?,
        };
        Ok(actor)
    }
}

#[async_trait]
impl Actor for MjProtocolHandler {
    type Msg = MjProtocolHandlerMessage;
    type State = MjProtocolHandlerState;
    type Arguments = ();

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting protocol handler");
        Ok(MjProtocolHandlerState {})
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MjProtocolHandlerMessage::Fetch(url, reply) => {
                let content = Self::dispatch(url).await.unwrap();
                let _ = reply.send(content);
            }
        }
        Ok(())
    }
}
