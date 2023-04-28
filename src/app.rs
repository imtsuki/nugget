use std::cell::RefCell;

use crate::Result;

use winit::{
    event::{Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
};

use crate::Renderer;
use crate::Resources;

#[derive(Debug)]
pub enum AppEvent {
    LoadResourcesRequest { path: String },
    LoadResourcesResponse(Result<Resources>),
}

thread_local! {
    pub static EVENT_LOOP_PROXY: RefCell<Option<EventLoopProxy<AppEvent>>> = RefCell::new(None);
}

pub async fn run(
    window: winit::window::Window,
    event_loop: EventLoop<AppEvent>,
    line: bool,
) -> Result<()> {
    let size = window.inner_size();

    let mut renderer = Renderer::new(&window, size.width, size.height, line).await?;

    #[allow(unused_variables)]
    let proxy = event_loop.create_proxy();

    let event_handler = move |event: Event<AppEvent>,
                              _: &EventLoopWindowTarget<AppEvent>,
                              control_flow: &mut ControlFlow| {
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
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, phase, .. },
                ..
            } => {
                tracing::debug!(?delta, ?phase);
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                renderer.rotate_camera(x, y);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::TouchpadMagnify { delta, .. },
                ..
            } => {
                tracing::debug!(?delta);
                renderer.zoom_camera(delta as f32);
                window.request_redraw();
            }
            Event::UserEvent(event) => {
                tracing::info!(?event, "received user event");
                match event {
                    AppEvent::LoadResourcesRequest { path } => {
                        #[cfg(target_arch = "wasm32")]
                        wasm_bindgen_futures::spawn_local(async {
                            let resources = Resources::load_gltf(path).await;
                            let _ =
                                crate::wasm::send_event(AppEvent::LoadResourcesResponse(resources));
                        });
                        // TODO: move this to a separate thread
                        #[cfg(not(target_arch = "wasm32"))]
                        pollster::block_on(async {
                            let resources = Resources::load_gltf(path).await;
                            let _ = proxy.send_event(AppEvent::LoadResourcesResponse(resources));
                        });
                    }
                    AppEvent::LoadResourcesResponse(Ok(resources)) => {
                        renderer.load_resources(resources);
                        window.request_redraw();
                    }
                    AppEvent::LoadResourcesResponse(Err(err)) => {
                        tracing::error!(?err, "failed to load resources");
                    }
                }
            }
            _ => {}
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run(event_handler);

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn(event_handler);
    }

    #[allow(unreachable_code)]
    Ok(())
}
