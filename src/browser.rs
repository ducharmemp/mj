use ractor::{async_trait, cast, Actor, ActorProcessingErr, ActorRef};

use crate::dom::{MjDom, MjDomMessage};

pub struct NavigateTo(pub String);

pub struct MjBrowser;

pub struct MjBrowserState {
    pub dom: ActorRef<MjDomMessage>,
}

#[async_trait]
impl Actor for MjBrowser {
    type Msg = NavigateTo;
    type State = MjBrowserState;
    type Arguments = ();

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        let (dom, _) = Actor::spawn_linked(None, MjDom, (), myself.into()).await?;
        Ok(MjBrowserState { dom })
    }
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let content = Box::new(reqwest::get(message.0).await?.text().await?);
        cast!(state.dom, MjDomMessage::Parse(content))?;

        Ok(())
    }
}
