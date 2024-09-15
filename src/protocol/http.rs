use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use reqwest::Response;
use tokio::{io::AsyncReadExt, sync::OnceCell};
use tracing::{event, instrument, Level};
use url::Url;

use super::MjProtocolMessage;

pub struct MjHttpProtocolHandler;

pub struct MjHttpProtocolHandlerState {
    response: OnceCell<Response>,
}

#[async_trait]
impl Actor for MjHttpProtocolHandler {
    type Msg = MjProtocolMessage;
    type State = MjHttpProtocolHandlerState;
    type Arguments = Url;

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting http protocol handler");
        let response = reqwest::get(args).await?;
        let container = OnceCell::new();
        container.set(response);

        Ok(MjHttpProtocolHandlerState {
            response: container,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MjProtocolMessage::Read(reply) => {
                let response = state.response.take().expect("Expected response");
                let response_body = response.text().await?;
                reply.send(response_body)?;
                myself.stop(None);
            }
            MjProtocolMessage::Write => todo!(),
        }
        Ok(())
    }
}
