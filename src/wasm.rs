use std::path;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use futures::TryFutureExt;

use anyhow::anyhow;

use winit::{dpi::LogicalSize, event_loop::EventLoopBuilder, platform::web::WindowExtWebSys};

use crate::app::AppEvent;
use crate::Result;

#[wasm_bindgen(start)]
pub fn wasm_start() -> Result<(), JsError> {
    wasm_main().map_err(|error| JsError::new(&error.to_string()))
}

#[wasm_bindgen(js_name = loadModel)]
pub fn load_model(path: Option<String>) -> Result<(), JsError> {
    send_event(AppEvent::LoadModelRequest {
        path: path.ok_or_else(|| JsError::new("No path provided"))?,
    })
}

pub fn send_event(event: AppEvent) -> Result<(), JsError> {
    crate::app::EVENT_LOOP_PROXY.with_borrow(|proxy| {
        proxy
            .as_ref()
            .ok_or_else(|| {
                JsError::new(
                    "No EventLoopProxy. Did you call wasm_main() before send_custom_event()?",
                )
            })?
            .send_event(event)?;

        Ok(())
    })
}

fn wasm_main() -> Result<()> {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();

    let window = winit::window::WindowBuilder::new()
        .with_title(env!("CARGO_PKG_NAME"))
        .with_inner_size(LogicalSize::new(800.0, 600.0))
        .build(&event_loop)?;

    let proxy = event_loop.create_proxy();

    crate::app::EVENT_LOOP_PROXY.set(Some(proxy));

    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| {
            document
                .get_element_by_id("canvas-container")
                .or_else(|| document.body().map(|body| body.into()))
        })
        .and_then(|container| container.append_child(&window.canvas()).ok())
        .ok_or_else(|| anyhow!("Failed to append canvas to body"))?;

    wasm_bindgen_futures::spawn_local(crate::app::run(window, event_loop, false).unwrap_or_else(
        |err| {
            tracing::error!(?err);
            ()
        },
    ));
    Ok(())
}

pub async fn fetch<P>(url: P) -> Result<Response, JsValue>
where
    P: AsRef<path::Path>,
{
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url.as_ref().to_string_lossy(), &opts)?;

    let window = web_sys::window().ok_or_else(|| JsError::new("no global `window` exists"))?;

    let response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .dyn_into()
        .expect("Response object");

    Ok(response)
}

async fn fetch_gltf<P: AsRef<path::Path>>(path: P) -> Result<gltf::Gltf, JsValue> {
    let response = fetch(&path).await?;
    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let gltf_data = js_sys::Uint8Array::new(&array_buffer).to_vec();
    let gltf =
        gltf::Gltf::from_slice(&gltf_data).map_err(|err| JsValue::from_str(&err.to_string()))?;
    Ok(gltf)
}

pub async fn import_gltf<P: AsRef<path::Path>>(
    path: P,
) -> Result<(gltf::Document, Vec<Vec<u8>>, Vec<web_sys::ImageBitmap>), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsError::new("no global `window` exists"))?;

    let base = path.as_ref().parent().unwrap_or_else(|| path.as_ref());

    let gltf::Gltf { document, mut blob } = fetch_gltf(&path).await?;

    let mut buffers = Vec::new();

    for buffer in document.buffers() {
        let data = match buffer.source() {
            gltf::buffer::Source::Uri(uri) => {
                let response = if uri.starts_with("data:") {
                    tracing::debug!("Fetching buffer from data URI");
                    fetch(uri).await
                } else {
                    let uri = base.join(uri);
                    tracing::debug!("Fetching buffer from {:?}", uri);
                    fetch(uri).await
                }?;
                let array_buffer = JsFuture::from(response.array_buffer()?).await?;
                js_sys::Uint8Array::new(&array_buffer).to_vec()
            }
            gltf::buffer::Source::Bin => blob.take().ok_or_else(|| {
                JsValue::from_str("Buffer source is bin, but no blob was provided")
            })?,
        };

        tracing::info!("Fetched buffer of size {}", data.len());

        buffers.push(data);
    }

    let mut images = Vec::new();

    for image in document.images() {
        let image_bitmap = match image.source() {
            gltf::image::Source::Uri { uri, mime_type: _ } => {
                let response = if uri.starts_with("data:") {
                    tracing::debug!("Fetching image from data URI");
                    fetch(uri).await
                } else {
                    let uri = base.join(uri);
                    tracing::debug!("Fetching image from {:?}", uri);
                    fetch(uri).await
                }?;

                let blob = JsFuture::from(response.blob()?)
                    .await?
                    .dyn_into()
                    .expect("Blob object");

                let image_bitmap = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?)
                    .await?
                    .dyn_into::<web_sys::ImageBitmap>()
                    .expect("ImageBitmap object");

                tracing::debug!(width = image_bitmap.width(), height = image_bitmap.height());

                image_bitmap
            }
            gltf::image::Source::View {
                view: _,
                mime_type: _,
            } => todo!(),
        };

        images.push(image_bitmap);
    }

    Ok((document, buffers, images))
}
