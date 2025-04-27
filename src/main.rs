mod renderer_backend;

use futures::executor::block_on;
use renderer_backend::pipeline_builder::PipelineBuilder;
use std::sync::Arc;
use winit::dpi::PhysicalSize;

use wgpu::{
    Adapter, RenderPipeline, RequestAdapterOptions, RequestAdapterOptionsBase, SurfaceTarget,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use wgpu::{Device, Instance, Queue, Surface, SurfaceConfiguration};

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    instance: Option<Instance>,
    surface: Option<Surface<'a>>,
    device: Option<Device>,
    queue: Option<Queue>,
    config: Option<SurfaceConfiguration>,
    size: (u32, u32),
    render_pipeline: Option<RenderPipeline>,
}

enum CustomEvent {
    Timer,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn get_surface(&self) -> &Surface<'a> {
        self.surface.as_ref().unwrap()
    }

    fn get_instance(&self) -> &Instance {
        self.instance.as_ref().unwrap()
    }

    fn get_device(&self) -> &Device {
        self.device.as_ref().unwrap()
    }

    fn get_queue(&self) -> &Queue {
        self.queue.as_ref().unwrap()
    }

    fn get_config(&self) -> &SurfaceConfiguration {
        self.config.as_ref().unwrap()
    }

    fn get_config_mut(&mut self) -> &mut SurfaceConfiguration {
        self.config.as_mut().unwrap()
    }

    async fn handle_adapter(
        &self,
        adapter_descriptor: &RequestAdapterOptions<'a, 'a>,
    ) -> (Adapter, Device, Queue) {
        let adapter = self
            .instance
            .as_ref()
            .unwrap()
            .request_adapter(&adapter_descriptor)
            .await
            .unwrap();
        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            label: Some("Device"),
            ..Default::default()
        };
        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();
        (adapter, device, queue)
    }

    fn set_window(&mut self, window: Window) {
        self.window = Some(Arc::new(window));
    }

    fn get_window(&self) -> Arc<Window> {
        self.window.as_ref().unwrap().clone()
    }

    fn init(&mut self) {
        let window = self.get_window();
        let size = window.as_ref().inner_size();
        // INSTANCE
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };
        let instance = wgpu::Instance::new(&instance_descriptor);
        self.instance = Some(instance);

        // SURFACE INIT
        let surface = self.get_instance().create_surface(window).ok();
        self.surface = surface;

        // ADAPTER
        let adapter_descriptor = RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: self.surface.as_ref(),
            force_fallback_adapter: false,
        };

        let (adapter, device, queue) = block_on(self.handle_adapter(&adapter_descriptor));

        // SURFACE CONFIGURATION
        let surface_capabilities = self.surface.as_ref().unwrap().get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let mut pipeline_builder = PipelineBuilder::new();
        pipeline_builder.set_shader_module("shaders/shader.wgsl", "vs_main", "fs_main");
        pipeline_builder.set_pixel_format(config.format);
        let render_pipeline = pipeline_builder.build(&device);

        self.surface.as_ref().unwrap().configure(&device, &config);

        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.size = (size.width, size.height);
        self.render_pipeline = Some(render_pipeline);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let drawable = self.get_surface().get_current_texture()?;
        let image_view_descc = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descc);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };

        let mut command_encoder = self
            .get_device()
            .create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.25,
                    g: 0.0,
                    b: 0.5,
                    a: 0.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        };
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Renderpass"),
            color_attachments: &[Some(color_attachment)],
            ..Default::default()
        };

        {
            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            renderpass.set_pipeline(&self.render_pipeline.as_ref().unwrap());
            renderpass.draw(0..3, 0..1);
        }

        self.get_queue()
            .submit(std::iter::once(command_encoder.finish()));

        drawable.present();
        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = (new_size.width, new_size.height);
            let config = self.get_config_mut();
            config.width = new_size.width as u32;
            config.height = new_size.height as u32;

            self.get_surface()
                .configure(self.get_device(), self.get_config());
        }
    }

    fn update_surface(&mut self) {
        let window = self.get_window();
        let target = SurfaceTarget::from(window);
        let surface = self.get_instance().create_surface(target).ok();
        self.surface = surface;
    }
}

impl<'a> ApplicationHandler<CustomEvent> for App<'a> {
    // This is a common indicator that you can create a window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        self.set_window(window);
        self.init();
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        window_event: WindowEvent,
    ) {
        match window_event {
            WindowEvent::RedrawRequested => match self.render() {
                Ok(_) => {}
                Err(err) => {
                    println!("Error rendering: {}", err);
                }
            },
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                let new_inner_size = PhysicalSize::new(
                    (self.size.0 as f64 * scale_factor) as u32,
                    (self.size.1 as f64 * scale_factor) as u32,
                );

                let _ = inner_size_writer.request_inner_size(new_inner_size);
            }
            WindowEvent::Resized(size) => {
                self.update_surface();
                self.resize(size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, event: CustomEvent) {
        match event {
            CustomEvent::Timer => {
                println!("Timer event received");
            }
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        println!("Exiting! Goodbye!");
    }
}

fn main() {
    let event_loop = EventLoop::<CustomEvent>::with_user_event().build().unwrap();
    let mut state = App::new();

    let event_loop_proxy = event_loop.create_proxy();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(17));
        event_loop_proxy.send_event(CustomEvent::Timer).ok();
    });

    let _ = event_loop.run_app(&mut state);
}
