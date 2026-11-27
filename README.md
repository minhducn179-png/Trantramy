# Trantramy

Ứng dụng nghe nhạc và Studio karaoke theo phong cách đĩa than.

Nhánh hiện tại chứa bản **2.0.0 Rust native** trong [`src-native`](src-native/README.md). Bản native dùng egui/eframe, Rodio và SQLite. Nó không cần Node.js hoặc WebView khi chạy. Mã Svelte/Tauri cũ được giữ trong repo để đối chiếu và bảo toàn các thay đổi trước khi chuyển đổi.

**Trạng thái phát hành:** gói 2.0 đang được chuẩn bị. Trang [Releases](https://github.com/minhducn179-png/Trantramy/releases) có thể vẫn hiển thị bản cũ cho đến khi tag 2.0 được build và phát hành thành công. Không lấy tên tệp dưới đây làm bằng chứng gói đã được đăng.

## Gói theo nền tảng

| Hệ điều hành | Kiến trúc | Gói 2.0 dự kiến |
|---|---|---|
| Windows | x64 | `Trantramy-2.0.0-windows-x64.zip` |
| macOS | Apple Silicon | `Trantramy-2.0.0-macos-arm64.dmg` |
| macOS | Intel | `Trantramy-2.0.0-macos-x64.dmg` |
| Linux | x64 | `Trantramy-2.0.0-linux-x64.tar.gz` |

Windows/Linux cần giải nén toàn bộ gói. Giữ thư mục công cụ đi kèm ứng dụng. Trên macOS, kéo ứng dụng từ DMG vào Applications. Gói macOS chưa được notarize. Mỗi gói có tệp SHA-256 để kiểm tra tải về.

## Tính năng native

- Thư viện nhạc cục bộ, kéo thả/quét thư mục, yêu thích và danh sách phát.
- Tìm nguồn YouTube/SoundCloud, nhập danh sách Spotify, lưu bài và tải âm thanh bằng yt-dlp.
- Bộ phát Rust, hàng chờ, lặp/ngẫu nhiên, EQ 10 dải, preamp và giới hạn clipping.
- Studio karaoke, gọi bài theo lượt, ưu tiên/xóa bài, đổi bản beat hoặc bản có giọng.
- Lời theo câu hoặc từng chữ khi có mốc thật. Bản nháp có cảnh báo và tách khỏi lời đã xác nhận.
- Sửa LRC, xem trước kết quả AI, giữ nội dung đang sửa, kiểm soát quyền gửi dữ liệu cloud.
- Các bảng màu, đĩa than, cửa sổ Studio riêng, mini lyrics, Dynamic Island và phím media.

Ứng dụng không tự tách giọng, thu âm hoặc chấm điểm. yt-dlp và các công cụ nhận diện tùy chọn vẫn là chương trình bên ngoài. Nhạc online hiện tải vào tệp tạm trước khi phát, tối đa 512 MB. Spotify chỉ cung cấp danh sách để tìm nguồn âm thanh tương ứng.

## Build bản Rust

```sh
cargo run --manifest-path src-native/Cargo.toml
cargo test --manifest-path src-native/Cargo.toml --all-targets --locked
cargo clippy --manifest-path src-native/Cargo.toml --all-targets --locked -- -D warnings
cargo build --manifest-path src-native/Cargo.toml --release --locked
```

Đọc [hướng dẫn native](src-native/README.md) để biết thư viện hệ thống Linux, cách kiểm tra thiết bị âm thanh, dữ liệu dùng lại và cách đóng gói. [Workflow](.github/workflows/build.yml) build Windows x64, macOS ARM64/Intel và Linux x64. Bước phát hành chỉ chạy khi tất cả job build thành công.

Kiểm tra cục bộ trên Windows gồm Rust regression tests, Clippy và chương trình kiểm tra desktop/thiết bị âm thanh. Kiểm tra headless không thay thế kiểm tra UI thực tế. Trạng thái build macOS/Linux cần kiểm tra trên runner tương ứng.

## Mã giao diện cũ

`src/`, `tests/`, `package.json` và phần Tauri còn lại là mã phiên bản WebView trước đó. `npm run check`, `npm test` và `npm run tauri dev` vẫn dành cho mã cũ. Workflow phát hành 2.0 dùng Cargo trong `src-native`, không đóng gói frontend cũ.
