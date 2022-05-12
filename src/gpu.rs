use log::{error, info};

#[derive(Debug, Clone)]
pub struct GpuBuilder {
    backends: wgpu::Backends,
    device_type: wgpu::DeviceType,
    present_mode: wgpu::PresentMode,
    features: wgpu::Features,
    limits: wgpu::Limits,
}

impl GpuBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_backend(mut self, b: wgpu::Backend) -> Self {
        self.backends.insert(wgpu::Backends::from(b));
        self
    }

    pub fn with_device(mut self, d: wgpu::DeviceType) -> Self {
        self.device_type = d;
        self
    }

    pub fn with_feature(mut self, f: wgpu::Features) -> Self {
        self.features |= f;
        self
    }

    pub fn with_limits(mut self, l: wgpu::Limits) -> Self {
        self.limits = l;
        self
    }

    pub fn with_present_mode(mut self, p: wgpu::PresentMode) -> Self {
        self.present_mode = p;
        self
    }

    fn select_adapter(
        &self,
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> Option<wgpu::Adapter> {
        let adapters = instance.enumerate_adapters(self.backends);
        for a in adapters {
            let info = a.get_info();
            if info.device_type == self.device_type
                && self.backends.contains(wgpu::Backends::from(info.backend))
                && a.is_surface_supported(&surface)
            {
                return Some(a);
            }
        }
        error!(
            target: "gpu_build",
            "Any device with these requirements :\n\tBACKENDS: {0:?}, \n\tDEVICE TYPE: {1:?} has been found",
            self.backends, self.device_type
        );
        None
    }

    pub async fn build(&self, window: &winit::window::Window) -> Gpu {
        let instance = wgpu::Instance::new(self.backends);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = self.select_adapter(&instance, &surface).unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: self.features,
                    limits: self.limits.clone(),
                    label: Some("default_device"),
                },
                None,
            )
            .await
            .unwrap();

        info!("running on : {}", adapter.get_info().name);

        let window_size = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window_size.width,
            height: window_size.height,
            present_mode: self.present_mode,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };
        surface.configure(&device, &surface_config);

        Gpu {
            device,
            queue,
            surface,
            surface_config,
        }
    }
}

impl Default for GpuBuilder {
    fn default() -> Self {
        Self {
            backends: wgpu::Backends::PRIMARY,
            device_type: wgpu::DeviceType::DiscreteGpu,
            present_mode: wgpu::PresentMode::Fifo,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        }
    }
}

#[derive(Debug)]
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
}

impl Gpu {
    pub fn resize_surface(&mut self, new_size: (u32, u32)) {
        self.surface_config.width = new_size.0;
        self.surface_config.height = new_size.1;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn get_surface_texture_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }
}
