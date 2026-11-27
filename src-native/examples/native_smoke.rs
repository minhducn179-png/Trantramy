use eframe::{App, egui};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use trantramy::{
    app::{MusicApp, Page},
    audio,
    context::{AppHandle, Paths},
    db,
    player::Phase,
};

struct Smoke {
    app: MusicApp,
    output: PathBuf,
    step: usize,
    requested: bool,
    changed: Instant,
    _data: tempfile::TempDir,
}
impl App for Smoke {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let screenshots = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });
        for image in screenshots {
            let bytes = image.pixels.iter().flat_map(|c| c.to_array()).collect();
            image::RgbaImage::from_raw(image.width() as u32, image.height() as u32, bytes)
                .unwrap()
                .save(self.output.join(format!("native-{:02}.png", self.step)))
                .unwrap();
            self.step += 1;
            self.requested = false;
            self.changed = Instant::now();
        }
        if self.step == 10 {
            println!(
                "Native smoke passed: real device play/pause/seek/resume, ten rendered screenshots."
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.app.page = [
            Page::Home,
            Page::Library,
            Page::Lyrics,
            Page::Karaoke,
            Page::Settings,
        ][self.step % 5];
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(if self.step < 5 {
            egui::vec2(1100., 720.)
        } else {
            egui::vec2(520., 400.)
        }));
        self.app.update(ctx, frame);
        if !self.requested && self.changed.elapsed() > Duration::from_millis(350) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            self.requested = true;
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }
    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.app.on_exit(gl);
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let _entered = runtime.enter();
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Pass an output directory")?,
    );
    std::fs::create_dir_all(&output)?;
    let data = tempfile::tempdir()?;
    let wav = data.path().join("native-audio.wav");
    let mut writer = hound::WavWriter::create(
        &wav,
        hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )?;
    for n in 0..44100 * 8 {
        let sample = ((n as f32 * 440. * std::f32::consts::TAU / 44100.).sin() * 250.) as i16;
        writer.write_sample(sample)?;
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    std::fs::write(
        wav.with_extension("lrc"),
        "[00:00.00]Lời Việt có dấu\n[00:03.00]Hát cùng Trantramy\n[00:06.00]Giữ từng câu nhạc\n",
    )?;
    let services = AppHandle::open(Paths {
        data: data.path().join("data"),
        cache: data.path().join("cache"),
    })?;
    db::upsert_track(
        &services.state::<db::Db>().0.lock().unwrap(),
        wav.to_str().unwrap(),
        "Khúc hát tiếng Việt",
        "Trantramy",
        "Native smoke",
        8.,
        None,
    )?;
    let track = db::list_tracks(&services.state::<db::Db>().0.lock().unwrap(), "")?.remove(0);
    eframe::run_native(
        "Trantramy native verification",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100., 720.])
                .with_min_inner_size([520., 400.]),
            ..Default::default()
        },
        Box::new(move |cc| {
            let mut app = MusicApp::new(cc, services);
            app.player.start(vec![track.clone()], 0);
            app.audio.load(audio::prepare(&track.path)?, 0.02)?;
            std::thread::sleep(Duration::from_millis(250));
            assert!(
                app.audio.position() > 0.05,
                "Real audio output did not advance"
            );
            app.audio.sink.as_ref().unwrap().pause();
            app.audio.seek(2.)?;
            app.audio.sink.as_ref().unwrap().play();
            std::thread::sleep(Duration::from_millis(100));
            assert!(app.audio.position() >= 2., "Seek/resume failed");
            app.audio.sink.as_ref().unwrap().pause();
            app.player.phase = Phase::Paused;
            Ok(Box::new(Smoke {
                app,
                output,
                step: 0,
                requested: false,
                changed: Instant::now(),
                _data: data,
            }))
        }),
    )?;
    Ok(())
}
