use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use tracing::{event, instrument, Level};
use vello::Scene;
use winit::event_loop::EventLoopProxy;

pub struct MjWindowBackend;

pub enum MjWindowBackendMessage {}

pub struct MjWindowBackendState {
    scene: Scene,
    event_loop_proxy: EventLoopProxy<Scene>,
}

pub type MjWindowBackendArgs = EventLoopProxy<Scene>;

#[async_trait]
impl Actor for MjWindowBackend {
    type Msg = MjWindowBackendMessage;
    type State = MjWindowBackendState;
    type Arguments = MjWindowBackendArgs;

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting renderer");
        Ok(MjWindowBackendState {
            scene: Scene::new(),
            event_loop_proxy: args,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {};
        Ok(())
    }
}
