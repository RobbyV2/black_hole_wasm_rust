#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::wrong_self_convention)]

mod camera;
mod integrator;
mod physics;

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages, TextureViewDescriptor,
};

use camera::Camera;
use physics::{BlackHole, Disk, Planet};

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    log::info!("WASM module initialized");
}

#[wasm_bindgen]
pub struct BlackHoleRenderer {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,
    output_texture: wgpu::Texture,
    camera_buffer: wgpu::Buffer,
    disk_buffer: wgpu::Buffer,
    planet_buffer: wgpu::Buffer,
    background_texture: wgpu::Texture,
    camera: Camera,
    black_hole: BlackHole,
    disk: Disk,
    planet: Planet,
    start_time: f64,
    compute_width: u32,
    compute_height: u32,
}

#[wasm_bindgen]
impl BlackHoleRenderer {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<BlackHoleRenderer, JsValue> {
        log::info!("Initializing Black Hole Renderer");

        let width = canvas.width();
        let height = canvas.height();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        // For web, create surface using the canvas
        let surface = {
            #[cfg(target_arch = "wasm32")]
            {
                let target = wgpu::SurfaceTarget::Canvas(canvas.clone());
                instance
                    .create_surface(target)
                    .map_err(|e| JsValue::from_str(&format!("Failed to create surface: {:?}", e)))?
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // This path should never be taken since this is web-only code
                return Err(JsValue::from_str("This code only runs on wasm32 target"));
            }
        };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| JsValue::from_str("Failed to find an appropriate adapter"))?;

        log::info!("Adapter info: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    label: Some("Device"),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create device: {:?}", e)))?;

        log::info!("Device created successfully");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Display Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Render Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        log::info!("Render pipeline created");

        // Create compute pipeline - increase resolution to reduce pixelation
        let compute_width = 800u32;
        let compute_height = 600u32;

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create output texture
        let output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: compute_width,
                height: compute_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create camera buffer (align to 16 bytes)
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create disk buffer
        let disk_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Disk Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create planet buffer
        let planet_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Planet Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Load background texture from embedded data
        log::info!("Loading background texture...");
        let bg_bytes = include_bytes!("../../public/milkyway.jpg");
        log::info!("Background bytes loaded: {} bytes", bg_bytes.len());
        let bg_img = image::load_from_memory(bg_bytes)
            .map_err(|e| JsValue::from_str(&format!("Failed to load background: {}", e)))?
            .to_rgba8();
        let (bg_width, bg_height) = bg_img.dimensions();
        log::info!("Background texture decoded: {}x{}", bg_width, bg_height);

        log::info!("Creating GPU texture...");
        let background_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Background Texture"),
            size: wgpu::Extent3d {
                width: bg_width,
                height: bg_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        log::info!("Uploading texture data to GPU...");
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &background_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bg_img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * bg_width),
                rows_per_image: Some(bg_height),
            },
            wgpu::Extent3d {
                width: bg_width,
                height: bg_height,
                depth_or_array_layers: 1,
            },
        );
        log::info!("Background texture ready");

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: disk_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: planet_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &background_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        log::info!("Compute pipeline created");

        // Create sampler and render bind group
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let camera = Camera::new();
        let black_hole = BlackHole::sagittarius_a();
        let disk = Disk::default_accretion_disk();
        let planet = Planet::new_elliptical_orbit(7.0, 0.5, 0.4, 8.54e36);

        log::info!("Black hole: r_s = {} meters", black_hole.r_s);
        log::info!("Camera radius: {} meters", camera.radius);
        log::info!(
            "Planet semi-major axis: {} meters, eccentricity: {}",
            planet.semi_major_axis,
            planet.eccentricity
        );

        Ok(BlackHoleRenderer {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            render_bind_group,
            compute_pipeline,
            compute_bind_group,
            output_texture,
            camera_buffer,
            disk_buffer,
            planet_buffer,
            background_texture,
            camera,
            black_hole,
            disk,
            planet,
            start_time: js_sys::Date::now() / 1000.0,
            compute_width,
            compute_height,
        })
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        self.update_uniforms();

        let output = self.surface.get_current_texture().map_err(|e| {
            JsValue::from_str(&format!("Failed to acquire next swap chain: {:?}", e))
        })?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Note: clear_texture clears to (0,0,0,0) which is transparent
        // The compute shader will write opaque colors to all pixels

        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);

            let workgroup_count_x = (self.compute_width + 15) / 16;
            let workgroup_count_y = (self.compute_height + 15) / 16;
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        // Render pass - display the computed texture
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update_uniforms(&mut self) {
        use glam::Vec3;

        let pos = self.camera.position();
        let target = self.camera.target;
        let up = Vec3::Y;

        let forward = (target - pos).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward).normalize();

        let fov = 60.0f32;
        let aspect = self.config.width as f32 / self.config.height as f32;
        let tan_half_fov = (fov.to_radians() / 2.0).tan();

        let camera_data: Vec<f32> = vec![
            pos.x,
            pos.y,
            pos.z,
            0.0,
            right.x,
            right.y,
            right.z,
            0.0,
            up.x,
            up.y,
            up.z,
            0.0,
            forward.x,
            forward.y,
            forward.z,
            0.0,
            tan_half_fov,
            aspect,
            if self.camera.moving { 1.0 } else { 0.0 },
            0.0,
        ];

        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&camera_data));

        let disk_data: Vec<f32> = vec![
            self.disk.inner_radius,
            self.disk.outer_radius,
            0.0,
            self.disk.thickness,
        ];

        self.queue
            .write_buffer(&self.disk_buffer, 0, bytemuck::cast_slice(&disk_data));

        // Update planet orbit
        let current_time = js_sys::Date::now() / 1000.0;
        let elapsed_time = (current_time - self.start_time) as f32;
        self.planet.update(elapsed_time);

        let planet_data: Vec<f32> = vec![
            self.planet.position.x,
            self.planet.position.y,
            self.planet.position.z,
            self.planet.radius,
        ];

        self.queue
            .write_buffer(&self.planet_buffer, 0, bytemuck::cast_slice(&planet_data));
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            log::info!("Resized to {}x{}", width, height);
        }
        Ok(())
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        let old_az = self.camera.azimuth;
        let old_el = self.camera.elevation;
        self.camera.process_mouse_move(x, y);
        if self.camera.dragging {
            log::info!(
                "Mouse move: az {:.4} -> {:.4}, el {:.4} -> {:.4}",
                old_az,
                self.camera.azimuth,
                old_el,
                self.camera.elevation
            );
        }
    }

    pub fn on_mouse_button(&mut self, button: u8, pressed: bool, x: f64, y: f64) {
        self.camera.process_mouse_button(button, pressed, x, y);
    }

    pub fn on_wheel(&mut self, delta_y: f64) {
        self.camera.process_scroll(delta_y);
    }

    pub fn camera_info(&self) -> String {
        let pos = self.camera.position();
        format!(
            "Camera: pos=({:.2e}, {:.2e}, {:.2e}), radius={:.2e}m, az={:.2}, el={:.2}",
            pos.x, pos.y, pos.z, self.camera.radius, self.camera.azimuth, self.camera.elevation
        )
    }
}

const SHADER_SOURCE: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );

    var uv = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0)
    );

    var output: VertexOutput;
    output.position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
    output.uv = uv[in_vertex_index];
    return output;
}

@group(0) @binding(0) var compute_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(compute_texture, texture_sampler, input.uv);
}
"#;
