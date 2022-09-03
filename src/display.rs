use crate::gpu;
use std::borrow::Cow;
use wgpu::{
    PrimitiveState, RenderPassDescriptor, ImageCopyTexture, ImageDataLayout,
    ShaderModuleDescriptor, ColorTargetState, ColorWrites, RenderPipelineDescriptor,
    RenderPassColorAttachment, PipelineLayoutDescriptor, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, SamplerBindingType, ShaderStages, TextureSampleType, BindGroup,
    BindGroupEntry, SamplerDescriptor, FilterMode, BindingResource, util::DeviceExt,
    TextureDescriptor, TextureUsages, TextureViewDescriptor, Extent3d
};

use std::num::NonZeroU32;
use crate::chip8::Chip8;
use winit::{dpi::LogicalSize, platform::macos::WindowBuilderExtMacOS};

pub struct Chip8Display {
    pipeline: wgpu::RenderPipeline,
    context: gpu::RenderContext,
    bind_group: BindGroup,
    backing_texture: wgpu::Texture,
    time: f32,
    window: winit::window::Window,
}


impl Chip8Display {

    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Chip8Display {

        let overlay = image::load_from_memory(include_bytes!("../assets/frame.png")).unwrap();

        let window = winit::window::WindowBuilder::new()
            .with_inner_size(LogicalSize { width: overlay.width(), height: overlay.height() })
            .with_title("chip8-rs")
            .with_transparent(true)
            .with_titlebar_transparent(true)
            .build(&event_loop).unwrap();

        let context = futures::executor::block_on(crate::gpu::RenderContext::new(&window));
        let gpu::RenderContext {device, queue, ..} = &context;

        let vs = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shaders/display_vs.wgsl")))
        });

        let fs = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shaders/display_fs.wgsl")))
        });

        let target_state = ColorTargetState {
            blend: gpu::PREMULTIPLIED_ALPHA,
            format: context.surface_format,
            write_mask: ColorWrites::default()
        };

        let overlay_texture = device.create_texture_with_data(queue, &TextureDescriptor {
            label: Some("Overlay texture"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST
        }, &overlay.as_bytes());

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                    visibility: ShaderStages::FRAGMENT
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    visibility: ShaderStages::FRAGMENT
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                    visibility: ShaderStages::FRAGMENT
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    visibility: ShaderStages::FRAGMENT
                }
            ]
        });

        let display_sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let overlay_sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let pixels = Box::new([255u8; (Chip8::DISPLAY_WIDTH * Chip8::DISPLAY_HEIGHT) as usize]);

        let backing_texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Backing texture"),
                size: wgpu::Extent3d { width: Chip8::DISPLAY_WIDTH as u32, height: Chip8::DISPLAY_HEIGHT as u32, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING
            },
            pixels.as_ref()
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&display_sampler)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&backing_texture.create_view(&TextureViewDescriptor::default()))
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&overlay_sampler)
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&overlay_texture.create_view(&TextureViewDescriptor::default()))
                }

            ]
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            bind_group_layouts: &[&bind_group_layout],
            ..Default::default()
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            depth_stencil: None,
            multiview: None,
            layout: Some(&layout),
            multisample: wgpu::MultisampleState::default(),
            primitive: PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
            vertex: wgpu::VertexState { module: &vs, entry_point: "main", buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &fs, entry_point: "main", targets: &[Some(target_state)] }),
        });

        Chip8Display {
            pipeline,
            bind_group,
            backing_texture,
            context,
            window,
            time: 0.0
        }

    }

    pub fn window(&self) -> &winit::window::Window {
        return &self.window;
    }

    pub fn update(&mut self, pixels: &[u8]) {
        let context = &self.context;
        let current_surface = context.surface.get_current_texture().unwrap();
        let current_texture_view = current_surface.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let color_attachment = RenderPassColorAttachment {
            view: &current_texture_view,
            resolve_target: None,
            ops: gpu::LOAD_STORE_TRANSPARENT_OPS
        };

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        context.queue.write_texture(
            ImageCopyTexture {
                texture: &self.backing_texture,
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 }
            },
            &pixels,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(64),
                rows_per_image: NonZeroU32::new(32)
            },
            Extent3d { width: 64, height: 32, depth_or_array_layers: 1 }
        );

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(color_attachment)],
            label: None,
            depth_stencil_attachment: None
        });

        let pipeline = &self.pipeline;
        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
        drop(render_pass);

        let command_buffer = encoder.finish();
        context.queue.submit([command_buffer]);

        current_surface.present();
        self.time += 0.01666666;

    }
}
