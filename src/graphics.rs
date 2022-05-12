#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

impl Vertex2D {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 0,
                    offset: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 1,
                    offset: 8,
                },
            ],
        }
    }
}

pub fn vertex2d(pos: impl Into<[f32; 2]>, uv: impl Into<[f32; 2]>) -> Vertex2D {
    Vertex2D {
        pos: pos.into(),
        uv: uv.into(),
    }
}

pub const fn vertex2d_const(pos: [f32; 2], uv: [f32; 2]) -> Vertex2D {
    Vertex2D { pos, uv }
}

pub const QUAD: &[Vertex2D] = &[
    vertex2d_const([-0.5, 0.5], [0.0, 0.0]),
    vertex2d_const([-0.5, -0.5], [0.0, 1.0]),
    vertex2d_const([0.5, -0.5], [1.0, 1.0]),
    vertex2d_const([0.5, 0.5], [1.0, 0.0]),
];

pub const QUAD_INDICES: &[u32] = &[0, 1, 3, 1, 2, 3];

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 0,
                    offset: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 1,
                    offset: 16,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 2,
                    offset: 28,
                },
            ],
        }
    }
}

pub fn vertex(
    pos: impl Into<[f32; 4]>,
    normal: impl Into<[f32; 3]>,
    uv: impl Into<[f32; 2]>,
) -> Vertex {
    Vertex {
        pos: pos.into(),
        normal: normal.into(),
        uv: uv.into(),
    }
}

#[derive(Debug, Clone)]
pub struct TextureBuilder<'a> {
    data: &'a [u8],
    format: wgpu::TextureFormat,
    usages: wgpu::TextureUsages,
    address_mode: wgpu::AddressMode,
    min_filter: wgpu::FilterMode,
    mag_filter: wgpu::FilterMode,
    texture_desc: Option<wgpu::TextureDescriptor<'a>>,
    sampler_desc: Option<wgpu::SamplerDescriptor<'a>>,
}

impl<'a> Default for TextureBuilder<'a> {
    fn default() -> Self {
        Self {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usages: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            address_mode: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Nearest,
            mag_filter: wgpu::FilterMode::Linear,
            texture_desc: None,
            sampler_desc: None,
            data: &[],
        }
    }
}

impl<'a> TextureBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_format(mut self, f: wgpu::TextureFormat) -> Self {
        self.format = f;
        self
    }

    pub fn with_usages(mut self, u: wgpu::TextureUsages) -> Self {
        self.usages |= u;
        self
    }

    pub fn with_address_mode(mut self, m: wgpu::AddressMode) -> Self {
        self.address_mode = m;
        self
    }

    pub fn with_min_filter(mut self, ft: wgpu::FilterMode) -> Self {
        self.min_filter = ft;
        self
    }

    pub fn with_mag_filter(mut self, ft: wgpu::FilterMode) -> Self {
        self.mag_filter = ft;
        self
    }

    pub fn with_texture_desc(mut self, td: wgpu::TextureDescriptor<'a>) -> Self {
        self.texture_desc = Some(td);
        self
    }

    pub fn with_sampler_desc(mut self, sd: wgpu::SamplerDescriptor<'a>) -> Self {
        self.sampler_desc = Some(sd);
        self
    }

    pub fn with_data(mut self, data: &'a [u8]) -> Self {
        self.data = data;
        self
    }

    pub fn build(&self, dim: (u32, u32), device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
        let size = wgpu::Extent3d {
            width: dim.0,
            height: dim.1,
            depth_or_array_layers: 1,
        };

        let wgpu_texture = match &self.texture_desc {
            Some(desc) => device.create_texture(&desc),
            None => device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                format: self.format,
                dimension: wgpu::TextureDimension::D2,
                usage: self.usages,
                mip_level_count: 1,
                sample_count: 1,
                size,
            }),
        };

        let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = match &self.sampler_desc {
            Some(desc) => device.create_sampler(&desc),
            None => device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: self.address_mode,
                address_mode_v: self.address_mode,
                address_mode_w: self.address_mode,
                min_filter: self.min_filter,
                mag_filter: self.mag_filter,
                ..Default::default()
            }),
        };
        let texture = Texture {
            texture: wgpu_texture,
            sampler,
            view,
            size,
            texel_size: self.format.describe().block_size,
        };
        texture.upload_data(self.data, queue);

        texture
    }
}

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    texel_size: u8,
}

impl Texture {
    pub fn upload_data(&self, data: &[u8], queue: &wgpu::Queue) {
        if data.len() != 0 {
            queue.write_texture(
                wgpu::ImageCopyTextureBase {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(self.size.width * self.texel_size as u32),
                    rows_per_image: std::num::NonZeroU32::new(self.size.height),
                },
                self.size,
            );
        }
    }
}
