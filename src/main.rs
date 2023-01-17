use anyhow::{Error, Result};
use clap::Parser;
use nugget::{Model, Renderer};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

/// Who hates nuggets?
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the glTF model to load
    path: String,
    /// Whether to render in wireframe mode
    #[arg(short, long)]
    line: bool,
}

pub fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;
    pollster::block_on(async {
        let size = window.inner_size();

        let model = Model::load_gltf(args.path)?;

        let mut renderer =
            Renderer::new(&window, size.width, size.height, model, args.line).await?;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    // Reconfigure the surface with the new size
                    renderer.size_changed(size.width, size.height);
                    // On macOS the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => renderer.render(),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        });
        #[allow(unreachable_code)]
        Ok::<(), Error>(())
    })
    .map_err(|error| {
        tracing::error!(?error);
        error
    })?;
    Ok(())
}
