# Third-party components

Trantramy's native dependency versions are recorded in `Cargo.lock`. Dependencies retain their upstream licenses. Principal components include eframe/egui, Rodio, Symphonia, rusqlite/SQLite, Tokio, souvlaki, rfd, ureq and their transitive dependencies.

The bundled yt-dlp executable comes from the release and asset recorded in `sidecars.json`. The build verifies its SHA-256 before packaging. yt-dlp source code is available at https://github.com/yt-dlp/yt-dlp . Its standalone executables include third-party components and are distributed under the licensing terms described in the upstream repository's license section and release materials. The application does not claim that this external executable is Rust code.

Upstream source and license information:

- egui/eframe: https://github.com/emilk/egui
- Rodio: https://github.com/RustAudio/rodio
- Symphonia: https://github.com/pdeljanov/Symphonia
- rusqlite: https://github.com/rusqlite/rusqlite
- SQLite: https://www.sqlite.org/copyright.html
- Tokio: https://github.com/tokio-rs/tokio
- souvlaki: https://github.com/Sinono3/souvlaki
- rfd: https://github.com/PolyMeilex/rfd
- ureq: https://github.com/algesten/ureq

Whisper models and whisper.cpp are optional downloads through the application's model settings. They are not included in the base archive. AI provider services are external services governed by their own terms and data-consent settings.
