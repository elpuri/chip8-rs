use wgpu::{
    Backends, SurfaceConfiguration,
    BlendState, BlendComponent, BlendFactor, BlendOperation, DeviceDescriptor, TextureFormat
};
pub struct RenderContext {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub surface_format: wgpu::TextureFormat,
}

impl RenderContext {
    pub async fn new(window: &winit::window::Window) -> RenderContext {
        let instance = wgpu::Instance::new(Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let chosen_adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await.expect("Couldn't acquire adapter");

        let desc = DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::downlevel_defaults()
        };
        let (device, queue) = chosen_adapter.request_device(&desc, None).await.expect("Couldn't acquire device");
        let size = window.inner_size();

        let supported_formats = surface.get_supported_formats(&chosen_adapter);

        let surface_format = supported_formats.iter().find(|format| **format == TextureFormat::Bgra8Unorm).expect("Couldn't get rgba8unorm surface").clone();
//        let surface_format = supported_formats[0];
        let surface_config = SurfaceConfiguration {
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
        };
        surface.configure(&device, &surface_config);

        RenderContext {
            instance,
            adapter: chosen_adapter,
            device,
            queue,
            surface,
            surface_format
        }
    }
}


pub const PREMULTIPLIED_ALPHA: Option<wgpu::BlendState> = Some(BlendState {
    alpha: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add },
    color: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add },
});

pub const LOAD_STORE_BLACK_OPS: wgpu::Operations<wgpu::Color> = wgpu::Operations {
    store: true,
    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0}),
};

pub const LOAD_STORE_TRANSPARENT_OPS: wgpu::Operations<wgpu::Color> = wgpu::Operations {
    store: true,
    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0}),
};

