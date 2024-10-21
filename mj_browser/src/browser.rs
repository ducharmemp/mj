use std::{
    error::Error,
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, Instant},
};

use stakker::{actor, call, ret_shutdown, ActorOwn, LogFilter, LogLevel, Stakker};
use stakker_log::KvSingleLine;
use stakker_mio::{
    mio::{Events, Poll},
    MioPoll,
};
use url::Url;
use vello::{
    peniko::Color,
    util::{RenderContext, RenderSurface},
    wgpu, AaConfig, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
    window::Window,
};

use crate::webview::MjWebview;

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

pub struct MjBrowser<'b> {
    webview: ActorOwn<MjWebview>,
    stakker: Stakker,
    miopoll: MioPoll,
    render_context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    render_state: RenderState<'b>,
    scene: Scene,
}

impl MjBrowser<'_> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut stakker = Stakker::new(Instant::now());
        stakker.set_logger(LogFilter::all(LogLevel::all_levels()), |_core, line| {
            let translated = match line.level {
                LogLevel::Trace => log::Level::Trace,
                LogLevel::Debug => log::Level::Debug,
                LogLevel::Info => log::Level::Info,
                LogLevel::Warn => log::Level::Warn,
                LogLevel::Error => log::Level::Error,
                LogLevel::Off => return,
                LogLevel::Audit => log::Level::Trace,
                LogLevel::Open => log::Level::Info,
                LogLevel::Close => log::Level::Info,
                _ => unreachable!(),
            };
            log::log!(target: line.target, translated, "{} {}", line.fmt, KvSingleLine::new(line.kvscan, " ", ""));
        });
        let miopoll = MioPoll::new(&mut stakker, Poll::new()?, Events::with_capacity(1024), 0)?;
        let webview = actor!(
            stakker,
            MjWebview::init(Url::parse("https://www.example.com").unwrap()),
            ret_shutdown!(stakker)
        );
        Ok(Self {
            stakker,
            miopoll,
            webview,
            render_context: RenderContext::new(),
            renderers: vec![],
            render_state: RenderState::Suspended(None),
            scene: Scene::new(),
        })
    }
}

impl ApplicationHandler for MjBrowser<'_> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        self.miopoll.poll(Duration::from_secs(0)).unwrap();
        self.stakker.run(Instant::now(), false);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.render_state else {
            panic!("Window is already active");
        };
        let window = cached_window.take().unwrap_or_else(|| {
            let attr = Window::default_attributes()
                .with_inner_size(LogicalSize::new(1044, 800))
                .with_resizable(true)
                .with_title("MJ");
            event_loop.create_window(attr).unwrap().into()
        });

        window.request_redraw();
        let size = window.inner_size();
        let surface_future = self.render_context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.render_context.devices.len(), || None);
        self.renderers[surface.dev_id].get_or_insert_with(|| {
            Renderer::new(
                &self.render_context.devices[surface.dev_id].device,
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
        self.render_state = RenderState::Active(ActiveRenderState { window, surface });

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Ignore the event (return from the function) if
        //   - we have no render_state
        //   - OR the window id of the event doesn't match the window id of our render_state
        //
        // Else extract a mutable reference to the render state from its containing option for use below
        let render_state = match &mut self.render_state {
            RenderState::Active(state) if state.window.id() == window_id => state,
            _ => return,
        };

        match event {
            // Exit the event loop when a close is requested (e.g. window's close button is pressed)
            WindowEvent::CloseRequested => event_loop.exit(),

            // Resize the surface when the window is resized
            WindowEvent::Resized(size) => {
                self.render_context.resize_surface(
                    &mut render_state.surface,
                    size.width,
                    size.height,
                );
                call!([self.webview], set_content_area(size.width, size.height));
                render_state.window.request_redraw();
            }

            // This is where all the rendering happens
            WindowEvent::RedrawRequested => {
                // Get the RenderSurface (surface + config)
                call!([self.webview], composite());
                self.scene.reset();
                let surface = &render_state.surface;

                // Get the window size
                let width = surface.config.width;
                let height = surface.config.height;

                // Get a handle to the device
                let device_handle = &self.render_context.devices[surface.dev_id];

                // Get the surface's texture
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                // Render to the surface's texture
                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_surface(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.scene,
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
            _ => {}
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if let RenderState::Active(state) = &self.render_state {
            self.render_state = RenderState::Suspended(Some(state.window.clone()));
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}
