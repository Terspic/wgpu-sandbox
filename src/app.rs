use std::time::{Duration, Instant};

use crate::gpu::{Gpu, GpuBuilder};
use futures_lite::future::block_on;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(feature = "imgui")]
use crate::imgui_support::*;

pub trait AppInstance {
    fn create(gpu: &Gpu) -> Self;
    fn events(&mut self, _event: &winit::event::WindowEvent) {}
    fn render(&self, gpu: &Gpu, frame: &wgpu::SurfaceTexture) {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                    view: &frame_view,
                    resolve_target: None,
                }],
                ..Default::default()
            });
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
    }
    fn update(&mut self, _gpu: &Gpu, _dt: Duration) {}
    fn destroy(&self) {}

    #[cfg(feature = "imgui")]
    fn on_imgui(&mut self, _ui: &imgui::Ui, _gpu: &Gpu, _dt: Duration) {}

    #[cfg(feature = "imgui")]
    fn imgui_setup(&mut self, ctx: &mut imgui::Context, window: &winit::window::Window) {
        ctx.set_ini_filename(None);
        let scale_factor = window.scale_factor() as f32;

        let font_size = 13.0;
        ctx.io_mut().font_global_scale = 1.0 / scale_factor;
        ctx.fonts().add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);
    }
}

#[derive(Debug, Clone)]
pub struct AppBuilder {
    name: String,
    dim: (u32, u32),
    gpu_builder: GpuBuilder,
    init_subscriber: bool,
    resizable: bool,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    pub fn with_gpu(mut self, gpu_builder: GpuBuilder) -> Self {
        self.gpu_builder = gpu_builder;
        self
    }

    pub fn with_dimension(mut self, widht: u32, height: u32) -> Self {
        self.dim = (widht, height);
        self
    }

    pub fn with_init_subscriber(mut self, value: bool) -> Self {
        self.init_subscriber = value;
        self
    }

    pub fn with_resizable(mut self, value: bool) -> Self {
        self.resizable = value;
        self
    }

    pub fn build(&self) -> App {
        if self.init_subscriber {
            wgpu_subscriber::initialize_default_subscriber(None);
        }

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(self.dim.0, self.dim.1))
            .with_title(self.name.as_str())
            .with_resizable(self.resizable)
            .build(&event_loop)
            .unwrap();

        let gpu = block_on(self.gpu_builder.build(&window));

        App {
            window,
            event_loop,
            gpu,
        }
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            name: String::from("default app"),
            dim: (640, 360),
            gpu_builder: GpuBuilder::default(),
            init_subscriber: true,
            resizable: true,
        }
    }
}

pub struct App {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    gpu: Gpu,
}

impl App {
    pub fn run<T: AppInstance + 'static>(mut self) {
        // build app
        let mut instance = T::create(&self.gpu);

        #[cfg(feature = "imgui")]
        // imgui setup
        let mut imgui_app_ctx = ImguiCtx::new(
            &self.window,
            &self.gpu.device,
            &self.gpu.queue,
            self.gpu.surface_config.format,
            &mut instance,
        );
        let mut last_frame = Instant::now();
        let mut dt = Duration::ZERO;

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { ref event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => (),
                        },
                        WindowEvent::Resized(size) => {
                            self.gpu.resize_surface((size.width, size.height));
                        }
                        _ => (),
                    }
                    instance.events(event)
                }
                Event::MainEventsCleared => {
                    let now = Instant::now();
                    dt = now - last_frame;
                    last_frame = now;

                    instance.update(&self.gpu, dt);

                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => match self.gpu.surface.get_current_texture() {
                    Ok(frame) => {
                        instance.render(&self.gpu, &frame);

                        #[cfg(feature = "imgui")]
                        {
                            imgui_app_ctx.ctx.io_mut().update_delta_time(dt);

                            // prepare imgui frame
                            imgui_app_ctx
                                .platform
                                .prepare_frame(imgui_app_ctx.ctx.io_mut(), &self.window)
                                .expect("Failed to prepare imgui frame");

                            let ui = imgui_app_ctx.ctx.frame();
                            instance.on_imgui(&ui, &self.gpu, dt);

                            let mut im_encoder = self.gpu.device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor {
                                    label: Some("imgui_command_encoder"),
                                },
                            );

                            let frame_view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());

                            {
                                let mut rpass =
                                    im_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: Some("imgui_render_pass"),
                                        color_attachments: &[wgpu::RenderPassColorAttachment {
                                            view: &frame_view,
                                            ops: wgpu::Operations {
                                                load: wgpu::LoadOp::Load,
                                                store: true,
                                            },
                                            resolve_target: None,
                                        }],
                                        depth_stencil_attachment: None,
                                    });

                                imgui_app_ctx
                                    .renderer
                                    .render(
                                        ui.render(),
                                        &self.gpu.queue,
                                        &self.gpu.device,
                                        &mut rpass,
                                    )
                                    .expect("Failed imgui rendering");
                            }

                            self.gpu.queue.submit(std::iter::once(im_encoder.finish()));
                        }
                        frame.present();
                    }
                    Err(wgpu::SurfaceError::Outdated) => {
                        println!("Surface outdated, skip frame")
                    }
                    Err(e) => eprintln!("{}", e),
                },
                Event::LoopDestroyed => {
                    instance.destroy();
                }
                _ => (),
            }
            #[cfg(feature = "imgui")]
            imgui_app_ctx
                .platform
                .handle_event(imgui_app_ctx.ctx.io_mut(), &self.window, &event);
        });
    }
}
