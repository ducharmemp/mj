use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use tracing::{event, instrument, Level};
use url::Url;

use crate::webview::{MjWebViewMessage, MjWebview};

pub struct NavigateTo(pub String);

pub struct MjBrowser;

pub struct MjBrowserState {
    pub webview: ActorRef<MjWebViewMessage>,
}

#[async_trait]
impl Actor for MjBrowser {
    type Msg = NavigateTo;
    type State = MjBrowserState;
    type Arguments = Option<Url>;

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting browser");
        event!(Level::DEBUG, "Spawning new default webview");
        let (webview, _) = Actor::spawn_linked(
            None,
            MjWebview,
            (args.unwrap_or(
                Url::parse("file:///home/matt/code/mj/resources/views/new.html")
                    .expect("Could not parse file path"),
            ),),
            myself.into(),
        )
        .await?;
        event!(Level::DEBUG, "Spawned new default webview");
        Ok(MjBrowserState { webview })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        Ok(())
    }
}
