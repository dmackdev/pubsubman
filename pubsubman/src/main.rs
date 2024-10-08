#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Optional override for the Pub/Sub Emulator project ID.
    #[arg(long)]
    emulator_project_id: Option<String>,
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let args = Args::parse();

    eframe::run_native(
        "pubsubman",
        Default::default(),
        Box::new(|cc| Ok(Box::new(pubsubman::App::new(cc, args.emulator_project_id)))),
    )
}
