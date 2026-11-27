use crate::context::AppHandle;
use std::{path::PathBuf, time::Duration};

pub fn sidecar_path(name: &str) -> Result<PathBuf, String> {
    let filename = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let directory = executable
        .parent()
        .ok_or("Không tìm thấy thư mục ứng dụng.")?;
    let mut candidates = vec![
        directory.join(&filename),
        directory.join("binaries").join(&filename),
        directory.join("../Resources/binaries").join(&filename),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(&filename),
    ];
    if cfg!(windows) {
        candidates.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../src-tauri/binaries/yt-dlp-x86_64-pc-windows-msvc.exe"),
        );
    }
    candidates
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| format!("Thiếu {filename}. Cài lại gói Trantramy đầy đủ."))
}
fn command() -> Result<tokio::process::Command, String> {
    let mut cmd = tokio::process::Command::new(sidecar_path("yt-dlp")?);
    cmd.kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    Ok(cmd)
}
pub async fn run_ytdlp(_app: &AppHandle, args: &[&str]) -> Result<String, String> {
    let output = tokio::time::timeout(
        Duration::from_secs(180),
        command()?
            .args([
                "--encoding",
                "utf-8",
                "--no-warnings",
                "--ignore-config",
                "--socket-timeout",
                "20",
                "--retries",
                "1",
            ])
            .args(args)
            .output(),
    )
    .await
    .map_err(|_| "yt-dlp quá thời gian chờ.")?
    .map_err(|e| format!("Không chạy được yt-dlp: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
pub async fn download_audio(
    app: &AppHandle,
    url: &str,
    dest: &std::path::Path,
    track_id: Option<i64>,
) -> Result<PathBuf, String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let template = dest.join("%(title).100B [%(id)s].%(ext)s");
    let mut child = command()?
        .args([
            "--encoding",
            "utf-8",
            "--ignore-config",
            "--no-warnings",
            "--no-playlist",
            "--newline",
            "--socket-timeout",
            "20",
            "--retries",
            "1",
            "--progress-template",
            "PROG:%(progress._percent_str)s",
            "-f",
            "bestaudio[ext=m4a]/bestaudio[ext=opus]/bestaudio[ext=webm]/bestaudio/best",
            "--print",
            "after_move:FILE:%(filepath)s",
            "-o",
        ])
        .arg(template)
        .arg("--")
        .arg(url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdout =
        BufReader::new(child.stdout.take().ok_or("Không mở được đầu ra yt-dlp.")?).lines();
    let mut stderr =
        BufReader::new(child.stderr.take().ok_or("Không mở được báo lỗi yt-dlp.")?).lines();
    let emit = |percent, status: &str| {
        let _ = app.emit(
            "download-progress",
            crate::ytdlp::DownloadProgress {
                url: url.into(),
                track_id,
                percent,
                status: status.into(),
            },
        );
    };
    emit(0.0, "downloading");
    let result = tokio::time::timeout(Duration::from_secs(1800), async {
        let mut path = None;
        let mut error = String::new();
        let (mut out_done, mut err_done) = (false, false);
        while !out_done || !err_done {
            tokio::select! {
                line = stdout.next_line(), if !out_done => match line.map_err(|e| e.to_string())? {
                    Some(line) => {
                        if let Some(s) = line.strip_prefix("PROG:") && let Ok(p) = s.trim().trim_end_matches('%').parse::<f64>() { emit(p, "downloading"); }
                        if let Some(s) = line.strip_prefix("FILE:") { path = Some(PathBuf::from(s.trim())); }
                    },
                    None => out_done = true,
                },
                line = stderr.next_line(), if !err_done => match line.map_err(|e| e.to_string())? {
                    Some(line) => { if error.len() < 16384 { error.push_str(&line); error.push('\n'); } },
                    None => err_done = true,
                },
            }
        }
        let status = child.wait().await.map_err(|e| e.to_string())?;
        match path { Some(p) if status.success() && p.is_file() => Ok(p), _ => Err(if error.is_empty() { "Tải nhạc thất bại.".into() } else { error }) }
    }).await.unwrap_or_else(|_| Err("Tải nhạc quá thời gian chờ.".into()));
    emit(
        if result.is_ok() { 100.0 } else { 0.0 },
        if result.is_ok() { "done" } else { "error" },
    );
    result
}
