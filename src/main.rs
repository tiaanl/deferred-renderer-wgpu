use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::WindowId,
};

mod app;
mod gizmos;
mod lights;
mod material;
mod mesh;
mod mesh_render_pipeline;
mod texture;

struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
}

enum AppState {
    Uninitialized,
    Initialized {
        window: Arc<winit::window::Window>,
        renderer: Renderer,
        app: app::App,
    },
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_title("Test wGPU")
                        .with_inner_size(LogicalSize::new(900, 600))
                        .with_resizable(false),
                )
                .expect("create window"),
        );

        let PhysicalSize { width, height } = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("request adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("request device");

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("create surface");

        let surface_caps = surface.get_capabilities(&adapter);

        // Find a sRGB surface format or use the first.
        let format = surface_caps
            .formats
            .iter()
            .find(|cap| cap.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let mut surface_config = surface
            .get_default_config(&adapter, width, height)
            .expect("surface get default configuration");
        surface_config.format = format;
        surface_config.present_mode = wgpu::PresentMode::AutoNoVsync;

        surface.configure(&device, &surface_config);

        let renderer = Renderer {
            device,
            queue,
            surface,
            surface_config,
        };

        let app = app::App::new(&renderer);

        *self = AppState::Initialized {
            window,
            renderer,
            app,
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        use winit::event::WindowEvent;

        match event {
            WindowEvent::CloseRequested => _event_loop.exit(),

            WindowEvent::Resized(size) => {
                let Self::Initialized {
                    window,
                    renderer,
                    app,
                } = self
                else {
                    return;
                };

                let PhysicalSize { width, height } = size;
                renderer.surface_config.width = width;
                renderer.surface_config.height = height;
                renderer
                    .surface
                    .configure(&renderer.device, &renderer.surface_config);

                app.resize(renderer);

                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                let Self::Initialized {
                    window,
                    renderer,
                    app,
                    ..
                } = self
                else {
                    return;
                };

                app.render(renderer);
                window.request_redraw();
            }

            WindowEvent::MouseInput { button, state, .. } => {
                let Self::Initialized { app, .. } = self else {
                    return;
                };

                match state {
                    ElementState::Pressed => app.on_mouse_down(button),
                    ElementState::Released => app.on_mouse_up(button),
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let Self::Initialized { app, .. } = self else {
                    return;
                };

                app.on_mouse_moved(position.x as f32, position.y as f32);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let Self::Initialized { app, .. } = self else {
                    return;
                };

                if let PhysicalKey::Code(key_code) = event.physical_key {
                    app.on_key_pressed(key_code)
                }
            }

            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("create event loop");
    let mut app = AppState::Uninitialized;
    event_loop.run_app(&mut app).expect("run app")
}
