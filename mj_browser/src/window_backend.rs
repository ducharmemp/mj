use std::{num::NonZeroUsize, sync::Arc};

use ractor::{async_trait, cast, Actor, ActorProcessingErr, ActorRef};
use taffy::{AvailableSpace, TaffyTree};
use tracing::{event, instrument, Level};
use vello::{
    kurbo::{Affine, RoundedRect, Stroke},
    peniko::Color,
    util::{RenderContext, RenderSurface},
    wgpu::{self},
    AaConfig, Renderer, RendererOptions, Scene,
};
use winit::{
    dpi::{PhysicalSize, Size},
    event_loop::EventLoopProxy,
    window::{Window, WindowId},
};

// Simple struct to hold the state of the renderer
pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<Window>,
}

enum RenderState<'s> {
    Active(ActiveRenderState<'s>),
    // Cache a window so that it can be reused when the app is resumed after being suspended
    Suspended(Option<Arc<Window>>),
}

pub struct MjWindowBackend;

pub enum MjWindowBackendMessage {
    WindowActivated(Arc<Window>),
    WindowResized(WindowId, PhysicalSize<u32>),
    WindowSuspended,
    RedrawWindow(WindowId),
    RenderScene((taffy::NodeId, TaffyTree)),
}

pub struct MjWindowBackendState<'s> {
    render_state: RenderState<'s>,
    render_context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    event_loop: EventLoopProxy<()>,
    scene: Scene,
}

pub type MjWindowBackendArgs = EventLoopProxy<()>;

#[async_trait]
impl Actor for MjWindowBackend {
    type Msg = MjWindowBackendMessage;
    type State = MjWindowBackendState<'static>;
    type Arguments = MjWindowBackendArgs;

    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting window renderer");
        Ok(MjWindowBackendState {
            event_loop: args,
            render_context: RenderContext::new(),
            renderers: Vec::new(),
            scene: Scene::new(),
            render_state: RenderState::Suspended(None),
        })
    }

    #[instrument(skip(self, myself, message, state))]
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MjWindowBackendMessage::RedrawWindow(window_id) => {
                let render_state = match &mut state.render_state {
                    RenderState::Active(state) if state.window.id() == window_id => state,
                    _ => return Ok(()),
                };
                event!(Level::INFO, "Drawing window");
                // Get the RenderSurface (surface + config)
                let surface = &render_state.surface;

                // Get the window size
                let width = surface.config.width;
                let height = surface.config.height;

                // Get a handle to the device
                let device_handle = &state.render_context.devices[surface.dev_id];

                // Get the surface's texture
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                // Render to the surface's texture
                state.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_surface(
                        &device_handle.device,
                        &device_handle.queue,
                        &state.scene,
                        &surface_texture,
                        &vello::RenderParams {
                            base_color: Color::BLACK, // Background color
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa16,
                        },
                    )
                    .expect("failed to render to surface");

                // Queue the texture to be presented on the surface
                surface_texture.present();

                device_handle.device.poll(wgpu::Maintain::Poll);
            }
            MjWindowBackendMessage::RenderScene((root_id, mut taffy)) => {
                let render_state = match &mut state.render_state {
                    RenderState::Active(state) => state,
                    _ => return Ok(()),
                };
                let surface = &render_state.surface;

                // Get the window size
                let width = surface.config.width;
                let height = surface.config.height;
                taffy.compute_layout(
                    root_id,
                    taffy::Size {
                        width: AvailableSpace::Definite(width as f32),
                        height: AvailableSpace::Definite(height as f32),
                    },
                )?;
                state.scene.reset();
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
                render_state.window.request_redraw();
            }
            MjWindowBackendMessage::WindowActivated(window) => {
                // Immediately request a redraw. This assists with creating surfaces below,
                // otherwise the surface might die unexpectedly
                window.request_redraw();
                // Create a vello Surface
                let size = window.inner_size();
                let surface = state
                    .render_context
                    .create_surface(
                        window.clone(),
                        size.width,
                        size.height,
                        wgpu::PresentMode::AutoVsync,
                    )
                    .await
                    .expect("Couild not create surface");
                // Create a vello Renderer for the surface (using its device id)
                state
                    .renderers
                    .resize_with(state.render_context.devices.len(), || None);
                state.renderers[surface.dev_id].get_or_insert_with(|| {
                    Renderer::new(
                        &state.render_context.devices[surface.dev_id].device,
                        RendererOptions {
                            surface_format: Some(surface.format),
                            use_cpu: false,
                            antialiasing_support: vello::AaSupport::all(),
                            num_init_threads: NonZeroUsize::new(1),
                        },
                    )
                    .expect("Couldn't create renderer")
                });

                // Save the Window and Surface to a state variable
                state.render_state = RenderState::Active(ActiveRenderState { window, surface });
            }
            MjWindowBackendMessage::WindowResized(window_id, size) => {
                // Ignore the event (return from the function) if
                //   - we have no render_state
                //   - OR the window id of the event doesn't match the window id of our render_state
                //
                // Else extract a mutable reference to the render state from its containing option for use below
                let render_state = match &mut state.render_state {
                    RenderState::Active(state) if state.window.id() == window_id => state,
                    _ => return Ok(()),
                };
                state.render_context.resize_surface(
                    &mut render_state.surface,
                    size.width,
                    size.height,
                )
            }
            MjWindowBackendMessage::WindowSuspended => {
                if let RenderState::Active(active_state) = &state.render_state {
                    state.render_state = RenderState::Suspended(Some(active_state.window.clone()));
                }
            }
        };
        Ok(())
    }
}
