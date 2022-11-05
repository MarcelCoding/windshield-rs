use wgpu::{
  Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Instance, Limits, LoadOp,
  Operations, PowerPreference, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor,
  RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError, TextureUsages,
  TextureViewDescriptor,
};
use wgpu::CompositeAlphaMode::Auto;
use winit::{
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::{Window, WindowBuilder},
};

struct State {
  surface: Surface,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
}

impl State {
  // Creating some of the wgpu types requires async
  // code
  async fn new(window: &Window) -> Self {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 +
    // Browser WebGPU
    let instance = Instance::new(Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(
        &DeviceDescriptor {
          features: Default::default(),
          // WebGL doesn't support all of wgpu's features, so if
          // we're building for the web we'll have to disable some.
          limits: if cfg!(target_arch = "wasm32") {
            Limits::downlevel_webgl2_defaults()
          } else {
            Limits::default()
          },
          label: None,
        },
        None, // Trace path
      )
      .await
      .unwrap();

    let config = SurfaceConfiguration {
      usage: TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_supported_formats(&adapter)[0],
      width: size.width,
      height: size.height,
      present_mode: PresentMode::Fifo,
      alpha_mode: Auto,
    };
    surface.configure(&device, &config);
    Self {
      surface,
      device,
      queue,
      config,
      size,
    }
  }

  fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }

  fn update(&mut self) {
    // todo!()
  }

  fn render(&mut self) -> Result<(), SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&TextureViewDescriptor::default());
    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });
    {
      let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color {
              r: 0.1,
              g: 0.2,
              b: 0.3,
              a: 1.0,
            }),
            store: true,
          },
        })],
        depth_stencil_attachment: None,
      });
    }

    // submit will accept anything that implements IntoIter
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}

pub async fn run() {
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  let mut state = State::new(&window).await;

  event_loop.run(move |event, _, control_flow| match event {
    Event::WindowEvent {
      ref event,
      window_id,
    } if window_id == window.id() => {
      if !state.input(event) {
        match event {
          WindowEvent::CloseRequested
          | WindowEvent::KeyboardInput {
            input:
            KeyboardInput {
              state: ElementState::Pressed,
              virtual_keycode: Some(VirtualKeyCode::Escape),
              ..
            },
            ..
          } => *control_flow = ControlFlow::Exit,
          WindowEvent::Resized(physical_size) => {
            state.resize(*physical_size);
          }
          WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
            // new_inner_size is &&mut so we have to dereference it twice
            state.resize(**new_inner_size);
          }
          _ => {}
        }
      }
    }
    Event::RedrawRequested(window_id) if window_id == window.id() => {
      state.update();
      match state.render() {
        Ok(_) => {}
        // Reconfigure the surface if lost
        Err(SurfaceError::Lost) => state.resize(state.size),
        // The system is out of memory, we should probably quit
        Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
        // All other errors (Outdated, Timeout) should be resolved by the next frame
        Err(e) => eprintln!("{:?}", e),
      }
    }
    Event::MainEventsCleared => {
      // RedrawRequested will only trigger once, unless we manually
      // request it.
      window.request_redraw();
    }
    _ => {}
  });
}
