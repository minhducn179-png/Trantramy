# Trantramy 2.0.0 — Rust native

Bản này chuyển giao diện, bộ phát âm thanh, hàng chờ và Studio sang Rust. Gói native chạy bằng egui/eframe và Rodio, không cần Node.js hoặc WebView. Backend Rust tiếp tục xử lý thư viện SQLite, nguồn trực tuyến và lời bài hát.

Gói phát hành dự kiến:

- Windows x64: `Trantramy-2.0.0-windows-x64.zip`.
- macOS Apple Silicon: `Trantramy-2.0.0-macos-arm64.dmg`.
- macOS Intel: `Trantramy-2.0.0-macos-x64.dmg`.
- Linux x64: `Trantramy-2.0.0-linux-x64.tar.gz`.

Giải nén toàn bộ gói Windows/Linux để giữ yt-dlp cạnh ứng dụng. macOS dùng DMG có ứng dụng được ký ad-hoc, chưa notarize. Mỗi gói có tệp SHA-256. yt-dlp được ghim phiên bản và kiểm checksum khi build.

Điểm thay đổi:

- Hàng chờ karaoke theo lượt, ưu tiên/xóa bài và đổi bản beat/có giọng giữ các bài đang chờ.
- EQ 10 dải, preamp, giới hạn clipping và hiển thị biên độ từ mẫu âm thanh.
- Giữ các bảng màu hiện có, đĩa than, cửa sổ Studio riêng, mini lyrics và Dynamic Island.
- Bản nháp lời và bản xem trước tách khỏi lời đã xác nhận. Nội dung đang sửa tự lưu cục bộ; lỗi ghi không đánh dấu đã lưu.
- AI tiếp tục yêu cầu quyền gửi dữ liệu và giới hạn ngân sách. Không tự tách giọng, ghi âm hay chấm điểm.

Giới hạn cần biết:

- Nhạc trực tuyến tải vào tệp tạm trước khi phát, tối đa 512 MB.
- Cài đặt giao diện WebView cũ chưa tự nhập sang native. Thư viện SQLite và cấu hình lời/AI tiếp tục dùng kho dữ liệu cũ.
- Cài model offline hiện chỉ hỗ trợ Windows x64. Nhập Spotify chỉ lấy danh sách và tìm nguồn âm thanh tương ứng.
- Chưa có cơ chế tự cập nhật native. Bộ cài có thể chưa có chữ ký được hệ điều hành tin cậy.

Kiểm tra cục bộ trên Windows: 58 Rust test của ứng dụng và một test checksum của trình đóng gói đạt, Clippy không có cảnh báo, kiểm tra thiết bị âm thanh thật đạt phát/tạm dừng/tua/phát tiếp. Chụp và kiểm tra các trang native ở cửa sổ rộng và hẹp. Các kiểm tra này không xác nhận chất lượng lời AI hoặc mọi nguồn trực tuyến. Trạng thái build các nền tảng cần đối chiếu workflow của đúng tag phát hành.
