use omarchy_scram::{
    app::ScramApp,
    packs,
    storage::{self, SessionLock},
};
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dir = storage::state_dir();
    let mut screenshot = None;
    let (mut width, mut height) = (800f32, 940f32);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("Omarchy Scram\n\nArrows / WASD / HJKL: move. Space: pause. F1: help.\n\n--state-dir PATH    Isolate local saves\n--screenshot PNG    Capture the native window and exit\n--width N          Window width (minimum 560)\n--height N         Window height (minimum 620)\n--validate-pack DIR Validate a character pack without a window\n--export-pack DIR   Create an editable Latch copy inside DIR\n--version          Print version");
                return Ok(());
            }
            "--version" => {
                println!("Omarchy Scram {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--validate-pack" => {
                let path = PathBuf::from(args.next().ok_or("Missing pack directory")?);
                let data = packs::load_folder(&path)?;
                println!(
                    "Valid character pack: {} ({} player frames, {} pursuer frames)",
                    data.manifest.name,
                    data.manifest.player.frames.len(),
                    data.manifest.pursuers.frames.len()
                );
                return Ok(());
            }
            "--export-pack" => {
                let path = PathBuf::from(args.next().ok_or("Missing destination directory")?);
                println!("{}", packs::export_template(&path)?.display());
                return Ok(());
            }
            "--state-dir" => dir = PathBuf::from(args.next().ok_or("Missing state directory")?),
            "--screenshot" => {
                screenshot = Some(PathBuf::from(args.next().ok_or("Missing screenshot path")?))
            }
            "--width" => {
                width = args.next().ok_or("Missing width")?.parse::<f32>()?;
                if !width.is_finite() {
                    return Err("Width must be finite".into());
                }
                width = width.clamp(560., 4096.);
            }
            "--height" => {
                height = args.next().ok_or("Missing height")?.parse::<f32>()?;
                if !height.is_finite() {
                    return Err("Height must be finite".into());
                }
                height = height.clamp(620., 4096.);
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    let _lock = SessionLock::acquire(&dir)?;
    let icon = image::load_from_memory_with_format(
        include_bytes!("../assets/packs/latch/icon.png"),
        image::ImageFormat::Png,
    )?
    .into_rgba8();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_min_inner_size([560., 620.])
            .with_app_id("io.github.tcballard.omarchy-scram")
            .with_icon(eframe::egui::IconData {
                rgba: icon.as_raw().clone(),
                width: icon.width() as u32,
                height: icon.height() as u32,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "Omarchy Scram",
        options,
        Box::new(move |cc| Ok(Box::new(ScramApp::new(&cc.egui_ctx, dir, screenshot)))),
    )?;
    Ok(())
}
