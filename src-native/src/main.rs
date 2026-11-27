#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("Trantramy {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let runtime = tokio::runtime::Runtime::new()?;
    let _runtime = runtime.enter();
    let services = trantramy::context::AppHandle::system().map_err(std::io::Error::other)?;
    let mut options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Trantramy 2.0")
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([520.0, 400.0]),
        ..Default::default()
    };
    if let Ok(icon) = image::load_from_memory(include_bytes!("../../src-tauri/icons/icon.png")) {
        let icon = icon.to_rgba8();
        options.viewport = options.viewport.with_icon(eframe::egui::IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_raw(),
        });
    }
    eframe::run_native(
        "Trantramy",
        options,
        Box::new(move |cc| Ok(Box::new(trantramy::app::MusicApp::new(cc, services)))),
    )?;
    Ok(())
}
