use std::borrow::Cow;

use winsafe::{co, gui, prelude::*};

pub fn main() {
    let wnd = gui::WindowMain::new(gui::WindowMainOpts {
        size: (300, 150),
        title: "wilistat".to_owned(),
        style: co::WS::MAXIMIZE | co::WS::TILEDWINDOW,
        ..Default::default()
    });
    let app = std::cell::RefCell::new(None);
    let prev = std::cell::RefCell::new(None);
    let wnd2 = wnd.clone();
    wnd.on().wm_paint(move || {
        let hwnd = wnd2.hwnd();
        let rect = hwnd.GetClientRect()?;
        if prev.borrow().is_some_and(|prev| prev == rect) {
            return Ok(());
        }
        *prev.borrow_mut() = Some(rect);
        let size = Size::new(rect.right as u32, rect.bottom as u32);
        app.borrow_mut()
            .get_or_insert_with(|| {
                pollster::block_on(run(
                    size,
                    wgpu_winsafe_compat::WindowMain {
                        hwnd: std::num::NonZero::new(hwnd.ptr() as isize).unwrap(),
                        hinstance: std::num::NonZero::new(hwnd.hinstance().ptr() as isize).unwrap(),
                    },
                ))
            })
            .redraw();
        Ok(())
    });
    wnd.run_main(None).unwrap();
}

struct Size {
    width: u32,
    height: u32,
}

impl Size {
    fn new(width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self { width, height }
    }
}

struct App<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    render_pipeline: wgpu::RenderPipeline,
    queue: wgpu::Queue,
}

impl App<'_> {
    fn redraw(&self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

async fn run<'a>(size: Size, wnd: wgpu_winsafe_compat::WindowMain) -> App<'a> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

    let surface = instance.create_surface(wnd).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
            required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                .using_resolution(adapter.limits()),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        })
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let config = surface
        .get_default_config(&adapter, size.width, size.height)
        .unwrap();
    surface.configure(&device, &config);
    App {
        surface,
        device,
        render_pipeline,
        queue,
    }
}
