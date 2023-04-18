#![feature(local_key_cell_methods)]

pub mod camera;
pub mod material;
pub mod model;
pub mod renderer;
pub mod resources;
pub mod scene;
pub mod texture;
pub mod uniform;
pub mod vertex;

mod ext;

pub mod app;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use anyhow::Result;
pub use model::Model;
pub use renderer::Renderer;
pub use resources::Resources;
