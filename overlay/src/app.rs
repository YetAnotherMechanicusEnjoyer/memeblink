use shared::models::OverlayAnchor;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId, WindowLevel};

#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowAttributesExtWayland;

use crate::events::RuntimeEvent;
use crate::platform::traits::NativeOverlay;
use crate::renderer::texture::MediaAsset;

pub struct MemeBlinkOverlayApp<O, T>
where
    O: NativeOverlay,
{
    platform_engine: O,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface_config: Option<wgpu::SurfaceConfiguration>,

    render_pipeline: Option<wgpu::RenderPipeline>,
    texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,
    meme_texture: Option<wgpu::Texture>,
    meme_bind_group: Option<wgpu::BindGroup>,
    current_texture_size: (u32, u32),

    active_asset: Option<MediaAsset>,
    active_anchor: Option<OverlayAnchor>,
    custom_x: Option<i32>,
    custom_y: Option<i32>,
    expires_at: Option<Instant>,
    _event_marker: PhantomData<T>,
}

impl<O, T> MemeBlinkOverlayApp<O, T>
where
    O: NativeOverlay,
{
    #[inline]
    pub fn new(platform_engine: O) -> Self {
        Self {
            platform_engine,
            window: None,
            surface: None,
            device: None,
            queue: None,
            surface_config: None,
            render_pipeline: None,
            texture_bind_group_layout: None,
            sampler: None,
            meme_texture: None,
            meme_bind_group: None,
            current_texture_size: (0, 0),
            active_asset: None,
            active_anchor: None,
            custom_x: None,
            custom_y: None,
            expires_at: None,
            _event_marker: PhantomData,
        }
    }

    fn ensure_wgpu(&mut self, window: Arc<Window>) {
        if self.device.is_some() {
            return;
        }

        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).expect("Impossible to find a compatible GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("MemeBlink WGPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )).expect("Impossible to create WGPU device");

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats[0];

        let alpha_mode = if capabilities.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else if capabilities.alpha_modes.contains(&wgpu::CompositeAlphaMode::PostMultiplied) {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else {
            wgpu::CompositeAlphaMode::Inherit
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("MemeBlink Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                r#"
                struct VertexOutput {
                    @builtin(position) position: vec4<f32>,
                    @location(0) tex_coords: vec2<f32>,
                }

                @vertex
                fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
                    var out: VertexOutput;
                    var pos = array<vec2<f32>, 6>(
                        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
                        vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0)
                    );
                    var tex = array<vec2<f32>, 6>(
                        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 0.0),
                        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0)
                    );
                    out.position = vec4<f32>(pos[index], 0.0, 1.0);
                    out.tex_coords = tex[index];
                    return out;
                }

                @group(0) @binding(0) var t_diffuse: texture_2d<f32>;
                @group(0) @binding(1) var s_diffuse: sampler;

                @fragment
                fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
                }
                "#,
            )),
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("MemeBlink Bind Group Layout"),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("MemeBlink Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("MemeBlink Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface_config = Some(config);
        self.render_pipeline = Some(render_pipeline);
        self.texture_bind_group_layout = Some(texture_bind_group_layout);
        self.sampler = Some(sampler);
        self.current_texture_size = (0, 0);
    }

    fn render_frame(&mut self) {
        let (Some(surface), Some(device), Some(queue), Some(window)) = (
            self.surface.as_ref(),
            self.device.as_ref(),
            self.queue.as_ref(),
            self.window.as_ref(),
        ) else {
            return;
        };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        if let Some(config) = &mut self.surface_config
            && (config.width != size.width || config.height != size.height) {
                config.width = size.width;
                config.height = size.height;
                surface.configure(device, config);
        }

        let surface_texture = match surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("MemeBlink Render Encoder"),
        });

        let clear_color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

        if let Some(asset) = self.active_asset.as_ref() {
            let frame = asset.current_frame();

            if self.meme_texture.is_none() || self.current_texture_size != (frame.width, frame.height) {
                let texture_size = wgpu::Extent3d {
                    width: frame.width,
                    height: frame.height,
                    depth_or_array_layers: 1,
                };
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: Some("meme_texture"),
                    view_formats: &[],
                });

                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: self.texture_bind_group_layout.as_ref().unwrap(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(self.sampler.as_ref().unwrap()),
                        },
                    ],
                    label: Some("meme_bind_group"),
                });

                self.meme_texture = Some(texture);
                self.meme_bind_group = Some(bind_group);
                self.current_texture_size = (frame.width, frame.height);
            }

            let mut meme_bytes = Vec::with_capacity((frame.width * frame.height * 4) as usize);
            for &p in &frame.pixels {
                let a = ((p >> 24) & 0xFF) as u8;
                let r = ((p >> 16) & 0xFF) as u8;
                let g = ((p >> 8) & 0xFF) as u8;
                let b = (p & 0xFF) as u8;
                meme_bytes.push(r);
                meme_bytes.push(g);
                meme_bytes.push(b);
                meme_bytes.push(a);
            }

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: self.meme_texture.as_ref().unwrap(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &meme_bytes,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(frame.width * 4),
                    rows_per_image: Some(frame.height),
                },
                wgpu::Extent3d {
                    width: frame.width,
                    height: frame.height,
                    depth_or_array_layers: 1,
                },
            );

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("MemeBlink Draw Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
                render_pass.set_bind_group(0, self.meme_bind_group.as_ref().unwrap(), &[]);
                render_pass.draw(0..6, 0..1);
            }
        } else {
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("MemeBlink Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

impl<O> ApplicationHandler<RuntimeEvent> for MemeBlinkOverlayApp<O, RuntimeEvent>
where
    O: NativeOverlay,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut window_attributes = WindowAttributes::default()
            .with_title("MemeBlink Overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop);

        #[cfg(target_os = "linux")]
        {
            window_attributes = window_attributes.with_name("memeblink", "");
        }

        if let Ok(new_window) = event_loop.create_window(window_attributes) {
            let _ = self.platform_engine.initialize_overlay(&new_window);
            let _ = self
                .platform_engine
                .configure_input_passthrough(&new_window, true);
            let window_arc = Arc::new(new_window);
            self.ensure_wgpu(window_arc.clone());
            self.window = Some(window_arc);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render_frame(),
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: RuntimeEvent) {
        match event {
            RuntimeEvent::InjectMeme {
                anchor,
                mut asset,
                duration,
                custom_x,
                custom_y,
            } => {
                asset.reset();
                let frame = asset.current_frame();

                if let Some(ref window) = self.window {
                    self.platform_engine
                        .update_anchor(
                            window,
                            anchor,
                            frame.width,
                            frame.height,
                            custom_x,
                            custom_y,
                        )
                        .ok();
                    window.request_redraw();
                }
                self.active_asset = Some(asset);
                self.active_anchor = Some(anchor);
                self.custom_x = custom_x;
                self.custom_y = custom_y;
                self.expires_at = Some(Instant::now() + duration);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(expires_at) = self.expires_at
            && Instant::now() >= expires_at
        {
            self.active_asset = None;
            self.active_anchor = None;
            self.custom_x = None;
            self.custom_y = None;
            self.expires_at = None;

            if let Some(window) = &self.window {
                self.platform_engine
                    .update_anchor(window, OverlayAnchor::TopLeft, 1, 1, None, None)
                    .ok();
                window.request_redraw();
            }
            return;
        }

        if let Some(asset) = &self.active_asset
            && let Some(ref window) = self.window
        {
            if let Some(anchor) = self.active_anchor {
                let frame = asset.current_frame();
                self.platform_engine
                    .update_anchor(
                        window,
                        anchor,
                        frame.width,
                        frame.height,
                        self.custom_x,
                        self.custom_y,
                    )
                    .ok();
            }
            window.request_redraw();
        }
    }
}
