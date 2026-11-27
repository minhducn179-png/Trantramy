# Trantramy 2.0 — Rust native

Ứng dụng dùng egui/eframe cho giao diện, Rodio cho âm thanh và SQLite cho thư viện. Bản native không chạy Svelte, JavaScript, Tauri hay WebView. yt-dlp và công cụ nhận diện giọng hát là công cụ bên ngoài, không được viết lại bằng Rust.

## Mở ứng dụng

- Windows: giải nén toàn bộ ZIP, mở `Trantramy.exe`. Giữ thư mục `binaries` cạnh ứng dụng.
- macOS: mở DMG theo đúng chip, kéo `Trantramy.app` vào Applications. Bản dựng chỉ có chữ ký ad-hoc, chưa notarize.
- Linux x64: giải nén `.tar.gz`, chạy `./trantramy` trong thư mục vừa giải nén. Cần môi trường desktop, ALSA và D-Bus. Trên Ubuntu: `sudo apt install libasound2 libdbus-1-3 libxkbcommon0 libegl1 libgl1`.

Có thể xem phiên bản bằng `Trantramy.exe --version` hoặc `./trantramy --version`.

## Dữ liệu và cách dùng

Dữ liệu ở thư mục hệ điều hành dành cho `com.trantramy.player`. Bản native tiếp tục dùng `dlc.db` và kho lời hiện có. Cài đặt native nằm trong `native-settings.json`. Cài đặt giao diện cũ chưa tự nhập vì trước đây nằm trong localStorage của WebView. Cấu hình và quyền gửi dữ liệu AI vẫn được đọc từ kho Rust hiện có.

Thêm thư mục hoặc kéo thả tệp âm thanh để quét thư viện. Trang Trực tuyến hỗ trợ tìm bài, dán liên kết và nhập danh sách Spotify. Nút Gọi bài thêm vào hàng chờ. Trong Studio, chọn bản beat hoặc bản có giọng từ kết quả tìm kiếm; ứng dụng không tự tách giọng. Khi hết lượt karaoke, ứng dụng chờ gọi thêm bài.

Chỉ tô từng chữ khi nguồn có mốc từng chữ. Bản nháp chưa xác nhận luôn có cảnh báo. Tạo lời, sửa chính tả và căn theo giọng hát tạo bản xem trước trước khi lưu. Nội dung đang sửa được giữ trong `native-drafts`, tách khỏi lời đã xác nhận.

Phím cách: phát/tạm dừng. Trái/phải: tua theo bước. Ctrl+K: mở Studio. F11: toàn màn hình. Esc: đóng bảng phụ/thoát toàn màn hình. Các phím này không chiếm phím khi đang nhập văn bản.

## Giới hạn hiện tại

- Nhạc trực tuyến được tải vào tệp tạm trước khi phát, tối đa 512 MB. Chuyển bài trong lúc tải sẽ bỏ kết quả cũ khi hoàn tất.
- Nhập Spotify chỉ lấy danh sách và tìm nguồn âm thanh tương ứng, không phát luồng Spotify có DRM.
- Bộ cài model nhận diện offline hiện chỉ hỗ trợ Windows x64. AI cloud chỉ chạy khi có khóa và quyền gửi dữ liệu phù hợp.
- Cửa sổ mini và Dynamic Island dùng viewport native. Tính năng chống chụp màn hình phụ thuộc hệ điều hành.
- Chưa có bộ tự cập nhật native. Cập nhật bằng gói phát hành mới.

## Build và kiểm tra

```sh
cargo run --manifest-path src-native/Cargo.toml
cargo test --manifest-path src-native/Cargo.toml --all-targets --locked
cargo clippy --manifest-path src-native/Cargo.toml --all-targets --locked -- -D warnings
cargo build --manifest-path src-native/Cargo.toml --release --locked
```

Trên Linux, cài các gói phát triển `libasound2-dev libdbus-1-dev libxkbcommon-dev libgl1-mesa-dev libegl1-mesa-dev libx11-dev pkg-config` trước khi build. Cargo dùng Rust stable hỗ trợ edition 2024 và các crate trong Cargo.lock.

Kiểm tra thiết bị âm thanh và chụp các màn hình native, dùng dữ liệu tạm riêng:

```sh
cargo run --manifest-path src-native/Cargo.toml --example native_smoke -- .pi/native-captures
```

Lệnh này cần desktop và thiết bị âm thanh. Nó kiểm tra phát/tạm dừng/tua/phát tiếp, rồi chụp các trang ở cửa sổ rộng và hẹp. Đây không phải kiểm tra dịch vụ trực tuyến hay chất lượng căn lời AI.

Để đóng gói, đặt yt-dlp đã kiểm checksum từ `sidecars.json` vào `src-native/binaries/yt-dlp.exe` (Windows) hoặc `src-native/binaries/yt-dlp` (macOS/Linux), rồi chạy:

```sh
cargo run --manifest-path src-native/Cargo.toml --example package -- x86_64-pc-windows-msvc src-native/target/release dist
```

Thay target và thư mục release tương ứng khi đóng gói trên macOS/Linux. Trình đóng gói phải chạy trên hệ điều hành của gói đích. Nó kiểm checksum yt-dlp trước khi tạo gói. Tài liệu và giấy phép nằm trong `Contents/Resources` của ứng dụng Mac để được giữ khi kéo vào Applications. Workflow GitHub build Windows x64, macOS ARM64/Intel và Linux x64. Mọi job phải thành công trước bước phát hành.

Kiểm tra gói Windows sau giải nén, gồm checksum, phiên bản, sidecar và mở cửa sổ với dữ liệu tạm:

```powershell
powershell -File src-native/verify-windows-package.ps1
```

Thêm `-SkipGui` khi chỉ kiểm tra cấu trúc và executable trên CI. Script xóa dữ liệu tạm khi kết thúc.
