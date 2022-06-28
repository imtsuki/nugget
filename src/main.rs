use anyhow::{Error, Result};
use nugget::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;
    pollster::block_on(async {
        let renderer = Renderer::new(&window).await?;
        renderer.size_changed(window.inner_size().width, window.inner_size().height);

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
    })?;
    Ok(())
}
