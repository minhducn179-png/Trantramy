# Rust migration — 2.0

Bản native hiện nằm tại `src-native/`. Giao diện egui/eframe, bộ phát Rodio, EQ, hàng chờ, karaoke và LRC đều dùng Rust. Các module thư viện, scan, yt-dlp, Spotify, gợi ý và lời/AI được dùng lại từ `src-tauri/src` qua `#[path]`. Những module dùng môi trường ứng dụng nhận `AppHandle` native khi bật feature `native`; entrypoint native không liên kết Tauri/WebView.

Các phần mới:

- `src-native/src/app*.rs`: giao diện và phối hợp tác vụ, nhận kết quả bất đồng bộ qua message, loại kết quả cũ bằng generation/revision.
- `player.rs`: hàng chờ, karaoke theo lượt, thay nguồn, lặp/ngẫu nhiên và trạng thái phát.
- `audio.rs`: giải mã, phát, seek, EQ và waveform. Online tải vào tệp tạm trước khi phát, giới hạn 512 MB.
- `lrc.rs`: đọc mốc câu/chữ, offset, giữ tiếng Việt và sửa chính tả bảo toàn mốc.
- `context.rs`: kho dịch vụ dùng chung, SQLite, đường dẫn dữ liệu và sự kiện.
- `settings.rs`: cài đặt native, bảng màu, contrast và lưu cấu hình.
- `process.rs`: gọi công cụ yt-dlp đi kèm, timeout và tiến độ tải.
- `examples/native_smoke.rs`: kiểm tra thiết bị âm thanh và ảnh chụp các trang native.
- `examples/package.rs`: đóng gói ZIP/DMG/tar.gz theo nền tảng, kèm checksum và giấy phép dependency.

Không viết lại yt-dlp hoặc các engine nhận diện thành Rust. Cấu hình giao diện WebView chưa tự nhập từ localStorage. Mã frontend cũ vẫn được bảo toàn trong repo nhưng không được đóng gói vào bản native. Những phần này phải được mô tả rõ khi nói “toàn bộ ứng dụng Rust”.

Thư viện tiếp tục dùng database `dlc.db` trong kho `com.trantramy.player`. Cài đặt native và bản nháp lời nằm ở các tệp riêng. Quyền gửi dữ liệu AI vẫn qua backend hiện có.

Kiểm tra Windows quan sát trong phiên: build debug, Rust tests, Clippy với `-D warnings`, kiểm tra output âm thanh thật phát/tạm dừng/seek/phát tiếp và ảnh chụp UI rộng/hẹp. Build release và đóng gói cần kiểm tra riêng. macOS/Linux phải được build trên runner tương ứng trước khi công bố có gói.

Chưa đo so sánh CPU/RAM với bản cũ. Không khẳng định bản native nhanh hơn hoặc nhẹ hơn khi chưa có số đo.
