use stakker::{actor, call, ret_nop, ret_some_to, ActorOwn, CX};
use tracing::{event, instrument, Level};
use url::Url;

use crate::{dom::MjDom, protocol::handler::MjProtocolHandler};

pub struct MjWebview {
    url: Url,
    dom: ActorOwn<MjDom>,
    protocol_handler: ActorOwn<MjProtocolHandler>,
}

impl MjWebview {
    #[instrument(skip(cx))]
    pub fn init(cx: CX![], url: Url) -> Option<Self> {
        event!(Level::INFO, "Starting new webview");
        let dom = actor!(cx, MjDom::init(), ret_nop!());
        let protocol_handler = actor!(cx, MjProtocolHandler::init(), ret_nop!());
        let fetch_ret = ret_some_to!([dom], parse_document() as (String));
        call!([protocol_handler], fetch(url.clone(), fetch_ret));

        Some(Self {
            dom,
            url,
            protocol_handler,
        })
    }

    // #[instrument(skip(self, myself, state))]
    // async fn post_start(
    //     &self,
    //     myself: ActorRef<Self::Msg>,
    //     state: &mut Self::State,
    // ) -> Result<(), ActorProcessingErr> {
    //     event!(Level::INFO, "Navigating to initial view");
    //     let handler: ActorRef<MjProtocolHandlerMessage> =
    //         registry::where_is("mj:protocol_handler".to_string())
    //             .expect("Failed to find protocol handler")
    //             .into();
    //     let handle = call!(handler, MjProtocolHandlerMessage::Fetch, state.url.clone())?;
    //     event!(Level::INFO, "Fetched content");
    //     event!(Level::DEBUG, "Sending message to DOM to parse content");
    //     let dom_handle = state.dom.clone();
    //     forward!(
    //         handle,
    //         MjProtocolMessage::Read,
    //         dom_handle,
    //         MjDomMessage::ParseDocument
    //     )?;
    //     Ok(())
    // }
    //
    // #[instrument(skip(self))]
    // async fn handle(
    //     &self,
    //     myself: ActorRef<Self::Msg>,
    //     message: Self::Msg,
    //     state: &mut Self::State,
    // ) -> Result<(), ActorProcessingErr> {
    //     match message {
    //         MjWebViewMessage::DomUpdated => {
    //             event!(Level::INFO, "Rendering DOM");
    //             let dom = state.dom.clone();
    //             let renderer = where_is("mj:window_backend".to_string())
    //                 .expect("Could not find renderer")
    //                 .into();
    //             forward!(
    //                 dom,
    //                 MjDomMessage::IntoLayout,
    //                 renderer,
    //                 MjWindowBackendMessage::RenderScene
    //             )?;
    //         }
    //     };
    //     Ok(())
    // }
}
