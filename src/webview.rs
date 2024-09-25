use ractor::{
    async_trait, call, forward,
    registry::{self, where_is},
    Actor, ActorProcessingErr, ActorRef, RpcReplyPort,
};
use tracing::{event, instrument, Level};
use url::Url;

use crate::{
    dom::{MjDom, MjDomMessage},
    protocol::{handler::MjProtocolHandlerMessage, MjProtocolMessage},
    renderer::MjRendererMessage,
};

pub enum MjWebViewMessage {
    PrintDom(RpcReplyPort<String>),
    DomUpdated,
}
pub type MjWebViewArgs = (Url,);

pub struct MjWebview;

pub struct MjWebviewState {
    pub url: Url,
    pub dom: ActorRef<MjDomMessage>,
}

#[async_trait]
impl Actor for MjWebview {
    type Msg = MjWebViewMessage;
    type State = MjWebviewState;
    type Arguments = MjWebViewArgs;

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: MjWebViewArgs,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting new webview");
        let (dom, _) =
            Actor::spawn_linked(None, MjDom, myself.clone(), myself.clone().into()).await?;
        Ok(MjWebviewState { dom, url: args.0 })
    }

    #[instrument(skip(self, myself, state))]
    async fn post_start(
        &self,
        myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        event!(Level::INFO, "Navigating to initial view");
        let handler: ActorRef<MjProtocolHandlerMessage> =
            registry::where_is("mj:protocol_handler".to_string())
                .expect("Failed to find protocol handler")
                .into();
        let handle = call!(handler, MjProtocolHandlerMessage::Fetch, state.url.clone())?;
        event!(Level::INFO, "Fetched content");
        event!(Level::DEBUG, "Sending message to DOM to parse content");
        let dom_handle = state.dom.clone();
        forward!(
            handle,
            MjProtocolMessage::Read,
            dom_handle,
            MjDomMessage::ParseDocument
        )?;
        Ok(())
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MjWebViewMessage::PrintDom(reply) => {}
            MjWebViewMessage::DomUpdated => {
                let dom = state.dom.clone();
                let renderer = where_is("mj:renderer".to_string())
                    .expect("Could not find renderer")
                    .into();
                forward!(
                    dom,
                    MjDomMessage::IntoLayout,
                    renderer,
                    MjRendererMessage::RenderLayout
                )?;
            }
        };
        Ok(())
    }
}
