# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Product Purpose

DLC là ứng dụng nghe nhạc bằng Svelte trong cửa sổ desktop Tauri. Chế độ Phòng karaoke bổ sung việc gọi bài, xếp lượt và hát theo lời trong ứng dụng hiện có.

## Capabilities and Constraints

- Phòng karaoke có sân khấu riêng trong cửa sổ ứng dụng, nền theo hình nền/ảnh bìa và nút ẩn toàn bộ công cụ phụ. Người dùng có thể xem từng câu lớn hoặc toàn bộ lời tự cuộn, cùng công cụ tìm/tạo/sửa/lưu lời như bài thường.
- Phòng karaoke có sân khấu lời lớn, câu tiếp theo, tìm bài trong thư viện/YouTube và hàng đợi theo thứ tự. Karaoke thay trình diễn sóng nhạc độc lập ở thanh phát và lối mở nhanh trong Cài đặt.
- Trên trang thông tin bài hát, nút lời ở thanh phát cuộn tới câu hát trong phần lời nhúng, không mở khung lời thứ hai. Ở các trang khác, nút này vẫn bật/tắt khung lời.
- Người dùng yêu cầu giữ gọi bài, màn hình hát và chọn nguồn beat/có giọng. Người dùng loại chỉnh nhạc và micro khỏi phạm vi này.
- Không có tách giọng tự động, ghi âm hoặc chấm điểm trong phạm vi phòng karaoke.
- AI viết lyric luôn tự động thực hiện quy trình tiêu chuẩn đối chiếu âm thanh/ngữ cảnh, phân câu nhịp điệu, lấy mốc Whisper từng chữ và kiểm tra xuất lyric cho mọi model AI theo `LYRIC-STANDARDS.md`. Cài đặt được tinh giản tập trung vào chọn model AI viết lyric (Gemini, OpenAI, Groq, Offline) và các tính năng liên quan (ngôn ngữ, tự tạo lời, tra cứu mạng, dự phòng offline); các quy chuẩn và bước xử lý chất lượng luôn được tự động kích hoạt. Chỗ chưa rõ được đánh dấu và giữ trong cache bản nháp riêng, không coi là lyric hoàn chỉnh, không tự ghi đè lời cũ hoặc nội suy mốc bị thiếu.
- Nguồn beat/có giọng do người dùng kiểm tra và chọn. Thời gian lời dùng dữ liệu nguồn; nội suy theo câu phải được ghi là ước lượng.
- Việc bổ sung phòng karaoke giữ nguyên giao diện và các lựa chọn theme của ứng dụng.

## Evidence on Hand

`KARAOKE.md` ghi luồng sử dụng và giới hạn kiểm thử. `LYRICS.md` ghi nguồn lời và quyền riêng tư. `DESIGN.md` cùng CSS hiện có là hệ thống giao diện kế thừa.
