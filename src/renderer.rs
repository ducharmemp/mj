use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use taffy::{AvailableSpace, Size, TaffyTree};
use tracing::{event, instrument, Level};
use vello::{
    kurbo::{Affine, RoundedRect, Stroke},
    peniko::Color,
    Scene,
};
use winit::event_loop::EventLoopProxy;

pub struct MjRenderer;

pub enum MjRendererMessage {
    RenderLayout((taffy::NodeId, TaffyTree)),
}

pub struct MjRendererState {
    scene: Scene,
    event_loop_proxy: EventLoopProxy<Scene>,
}

pub type MjRendererArgs = EventLoopProxy<Scene>;

#[async_trait]
impl Actor for MjRenderer {
    type Msg = MjRendererMessage;
    type State = MjRendererState;
    type Arguments = MjRendererArgs;

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting renderer");
        Ok(MjRendererState {
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
        match message {
            MjRendererMessage::RenderLayout((root_id, mut taffy)) => {
                taffy.compute_layout(
                    root_id,
                    Size {
                        width: AvailableSpace::Definite(1044.0),
                        height: AvailableSpace::Definite(800.0),
                    },
                );
                state.scene.reset();
                println!("{:?}", taffy.print_tree(root_id));
                fn walk(scene: &mut Scene, taffy: &TaffyTree, node: taffy::NodeId) {
                    let layout = taffy.layout(node).unwrap();
                    let stroke = Stroke::new(6.0);
                    let rect = RoundedRect::new(
                        layout.location.x.into(),
                        layout.location.y.into(),
                        layout.size.width.into(),
                        layout.size.height.into(),
                        0.0,
                    );
                    let rect_stroke_color = Color::rgb(0.9804, 0.702, 0.5294);
                    scene.stroke(&stroke, Affine::IDENTITY, rect_stroke_color, None, &rect);
                    for child in taffy.children(node).unwrap() {
                        walk(scene, taffy, child);
                    }
                }
                walk(&mut state.scene, &taffy, root_id);
                state.event_loop_proxy.send_event(state.scene.clone());
            }
        };
        Ok(())
    }
}
