use anyhow::Result;
use clap::Parser;
use nugget::app;
use winit::event_loop::EventLoopBuilder;

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

    let event_loop = EventLoopBuilder::<app::AppEvent>::with_user_event().build();
    let window = winit::window::WindowBuilder::new()
        .with_title("nugget")
        .build(&event_loop)?;

    event_loop
        .create_proxy()
        .send_event(app::AppEvent::LoadModelRequest { path: args.path })?;

    pollster::block_on(nugget::app::run(window, event_loop, args.line)).map_err(|error| {
        tracing::error!(?error);
        error
    })?;

    Ok(())
}
