use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

fn copy(source: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dest.parent().ok_or("Invalid package path")?)?;
    fs::copy(source, dest)?;
    Ok(())
}
fn run(command: &str, args: &[&std::ffi::OsStr]) -> Result<(), Box<dyn std::error::Error>> {
    if !Command::new(command).args(args).status()?.success() {
        return Err(format!("{command} failed").into());
    }
    Ok(())
}
fn file_hash(file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = fs::File::open(file)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let size = input.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hash.update(&buffer[..size]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn verify_hash(file: &Path, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    if file_hash(file)? != expected {
        return Err("yt-dlp checksum mismatch".into());
    }
    Ok(())
}
fn checksum(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        file.with_file_name(format!(
            "{}.sha256",
            file.file_name().unwrap().to_string_lossy()
        )),
        format!(
            "{}  {}\n",
            file_hash(file)?,
            file.file_name().unwrap().to_string_lossy()
        ),
    )?;
    Ok(())
}
fn zip_dir(folder: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut zip = zip::ZipWriter::new(fs::File::create(dest)?);
    for entry in walkdir::WalkDir::new(folder) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let name = entry
                .path()
                .strip_prefix(folder.parent().unwrap())?
                .to_string_lossy()
                .replace('\\', "/");
            zip.start_file(
                name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o755),
            )?;
            std::io::copy(&mut fs::File::open(entry.path())?, &mut zip)?;
        }
    }
    zip.finish()?.flush()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: cargo run --example package -- <target-triple> <release-directory> <output-directory>".into());
    }
    let target = args[0].to_str().ok_or("Invalid target")?;
    let release = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    fs::create_dir_all(&output)?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let platform = match target {
        "x86_64-pc-windows-msvc" => "windows-x64",
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-unknown-linux-gnu" => "linux-x64",
        _ => return Err("Unsupported release target".into()),
    };
    let name = format!("Trantramy-{}-{platform}", env!("CARGO_PKG_VERSION"));
    let staging = tempfile::tempdir_in(&output)?;
    let folder = staging.path().join(&name);
    fs::create_dir_all(&folder)?;
    let windows = target.contains("windows");
    let mac = target.contains("apple");
    let exe = if windows {
        "trantramy.exe"
    } else {
        "trantramy"
    };
    let sidecar = if windows { "yt-dlp.exe" } else { "yt-dlp" };
    if windows != cfg!(target_os = "windows") || mac != cfg!(target_os = "macos") {
        return Err("Package on the target operating system".into());
    }
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("sidecars.json"))?)?;
    let asset = if windows {
        "windows"
    } else if mac {
        "macos"
    } else {
        "linux"
    };
    verify_hash(
        &root.join("binaries").join(sidecar),
        manifest["assets"][asset]["sha256"]
            .as_str()
            .ok_or("Missing sidecar checksum")?,
    )?;
    let resources = if mac {
        folder.join("Trantramy.app/Contents/Resources")
    } else {
        folder.clone()
    };
    let executable;
    if mac {
        let bundle = folder.join("Trantramy.app/Contents");
        executable = bundle.join("MacOS/Trantramy");
        copy(&release.join(exe), &executable)?;
        copy(
            &root.join("binaries").join(sidecar),
            &bundle.join("Resources/binaries").join(sidecar),
        )?;
        copy(
            &root.join("../src-tauri/icons/icon.icns"),
            &bundle.join("Resources/icon.icns"),
        )?;
        fs::write(
            bundle.join("Info.plist"),
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?><plist version=\"1.0\"><dict><key>CFBundleName</key><string>Trantramy</string><key>CFBundleDisplayName</key><string>Trantramy</string><key>CFBundleIdentifier</key><string>com.trantramy.player</string><key>CFBundleExecutable</key><string>Trantramy</string><key>CFBundleIconFile</key><string>icon.icns</string><key>CFBundleShortVersionString</key><string>{0}</string><key>CFBundleVersion</key><string>{0}</string><key>CFBundlePackageType</key><string>APPL</string><key>NSHighResolutionCapable</key><true/><key>LSMinimumSystemVersion</key><string>11.0</string><key>LSApplicationCategoryType</key><string>public.app-category.music</string></dict></plist>",
                env!("CARGO_PKG_VERSION")
            ),
        )?;
    } else {
        executable = folder.join(if windows {
            "Trantramy.exe"
        } else {
            "trantramy"
        });
        copy(&release.join(exe), &executable)?;
        copy(
            &root.join("binaries").join(sidecar),
            &folder.join("binaries").join(sidecar),
        )?;
        if !windows {
            copy(
                &root.join("../src-tauri/icons/128x128.png"),
                &folder.join("trantramy.png"),
            )?;
        }
    }
    let license_dir = resources.join("licenses");
    let metadata = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .output()?;
    if !metadata.status.success() {
        return Err("Cannot collect dependency license metadata".into());
    }
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout)?;
    let mut inventory = String::from("Dependency\tVersion\tLicense\tRepository\n");
    for package in metadata["packages"]
        .as_array()
        .ok_or("Missing dependency metadata")?
    {
        let name = package["name"].as_str().ok_or("Missing crate name")?;
        let version = package["version"].as_str().ok_or("Missing crate version")?;
        inventory.push_str(&format!(
            "{name}\t{version}\t{}\t{}\n",
            package["license"].as_str().unwrap_or("See source"),
            package["repository"].as_str().unwrap_or("")
        ));
        let manifest = Path::new(
            package["manifest_path"]
                .as_str()
                .ok_or("Missing manifest")?,
        );
        for entry in fs::read_dir(manifest.parent().ok_or("Missing crate root")?)? {
            let entry = entry?;
            let filename = entry.file_name();
            let upper = filename.to_string_lossy().to_uppercase();
            if entry.file_type()?.is_file()
                && ["LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE"]
                    .iter()
                    .any(|prefix| upper.starts_with(prefix))
            {
                copy(
                    &entry.path(),
                    &license_dir.join(format!("{name}-{version}")).join(filename),
                )?;
            }
        }
    }
    fs::write(resources.join("DEPENDENCIES.tsv"), inventory)?;
    copy(&root.join("README.md"), &resources.join("README.md"))?;
    copy(
        &root.join("THIRD_PARTY_NOTICES.md"),
        &resources.join("THIRD_PARTY_NOTICES.md"),
    )?;
    copy(
        &root.join("sidecars.json"),
        &resources.join("sidecars.json"),
    )?;
    if !windows {
        run("chmod", &["+x".as_ref(), executable.as_os_str()])?;
        let bundled_sidecar = if mac {
            folder.join("Trantramy.app/Contents/Resources/binaries/yt-dlp")
        } else {
            folder.join("binaries/yt-dlp")
        };
        run("chmod", &["+x".as_ref(), bundled_sidecar.as_os_str()])?;
    }
    let archive = if windows {
        let archive = output.join(format!("{name}.zip"));
        zip_dir(&folder, &archive)?;
        archive
    } else if mac {
        let app = folder.join("Trantramy.app");
        run(
            "codesign",
            &[
                "--force".as_ref(),
                "--deep".as_ref(),
                "--sign".as_ref(),
                "-".as_ref(),
                app.as_os_str(),
            ],
        )?;
        run(
            "codesign",
            &[
                "--verify".as_ref(),
                "--deep".as_ref(),
                "--strict".as_ref(),
                app.as_os_str(),
            ],
        )?;
        run(
            "ln",
            &[
                "-s".as_ref(),
                "/Applications".as_ref(),
                folder.join("Applications").as_os_str(),
            ],
        )?;
        let archive = output.join(format!("{name}.dmg"));
        run(
            "hdiutil",
            &[
                "create".as_ref(),
                "-volname".as_ref(),
                "Trantramy".as_ref(),
                "-srcfolder".as_ref(),
                folder.as_os_str(),
                "-format".as_ref(),
                "UDZO".as_ref(),
                "-ov".as_ref(),
                archive.as_os_str(),
            ],
        )?;
        archive
    } else {
        let archive = output.join(format!("{name}.tar.gz"));
        run(
            "tar",
            &[
                "-czf".as_ref(),
                archive.as_os_str(),
                "-C".as_ref(),
                staging.path().as_os_str(),
                name.as_ref(),
            ],
        )?;
        archive
    };
    checksum(&archive)?;
    println!("Created {}", archive.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_rejects_changed_or_missing_sidecars() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("yt-dlp");
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        fs::write(&file, b"abc").unwrap();
        verify_hash(&file, expected).unwrap();
        checksum(&file).unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("yt-dlp.sha256")).unwrap(),
            format!("{expected}  yt-dlp\n")
        );
        fs::write(&file, b"changed").unwrap();
        assert!(verify_hash(&file, expected).is_err());
        fs::remove_file(&file).unwrap();
        assert!(verify_hash(&file, expected).is_err());
    }
}
