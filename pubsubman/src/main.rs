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

    let native_options = eframe::NativeOptions {
        follow_system_theme: false,
        ..Default::default()
    };

    eframe::run_native(
        "pubsubman",
        native_options,
        Box::new(|cc| Box::new(pubsubman::App::new(cc, args.emulator_project_id))),
    )
}
