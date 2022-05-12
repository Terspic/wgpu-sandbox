use crate::prelude::AppInstance;

pub struct ImguiCtx {
    pub ctx: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    pub renderer: imgui_wgpu::Renderer,
}

impl ImguiCtx {
    pub fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        instance: &mut impl AppInstance,
    ) -> Self {
        let mut ctx = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut ctx);
        platform.attach_window(
            ctx.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        instance.imgui_setup(&mut ctx, window);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(&mut ctx, &device, &queue, renderer_config);

        Self {
            ctx,
            platform,
            renderer,
        }
    }
}
