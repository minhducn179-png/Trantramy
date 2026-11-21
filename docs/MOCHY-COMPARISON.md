# Mochy-Music và DLC: đối chiếu phần lời bài hát

Đối chiếu mã nguồn Mochy tại commit `eafbbe6d1c2a6590cbdfa045b42b572611bb2133`. Không chạy app Android hoặc đo hiệu năng hai app.

## Kết luận

Giữ Svelte/Tauri/Rust cho DLC trên Windows. Học cách chọn nguồn và hiệu chỉnh lyrics của Mochy, không chuyển cả app sang Kotlin/Android.

| Tiêu chí | Mochy | DLC hiện tại |
| --- | --- | --- |
| Nền tảng | README hướng dẫn cài Android, bản desktop để sau | App desktop dùng Tauri/Rust và Svelte |
| Nguồn lyrics | Có registry nhiều nhà cung cấp, chấm điểm lời theo ngôn ngữ và chất lượng đồng bộ | Trước thay đổi này chỉ tra LRCLIB, phụ đề, rồi AI; nay thêm LyricsPlus khi cho phép |
| Đồng bộ | Ưu tiên word → line → plain, có công cụ offset và tự căn theo âm thanh | Giữ mốc từng từ từ nguồn/Whisper; nay hỗ trợ `[offset:ms]` trong parser dùng chung và ô chỉnh trong editor |
| Khi không có lời từ dịch vụ | Có nhiều nguồn để thử | Có whisper.cpp offline và cloud theo cấu hình; vẫn có thể nhận sai chữ |
| Chọn nhầm phiên bản | Có chấm điểm metadata và ngôn ngữ | Giữ đối chiếu tên bài/nghệ sĩ và sai lệch thời lượng tối đa 3 giây; có thể bỏ qua kết quả đúng nếu metadata khác |

Đa nguồn không đồng nghĩa mọi bài đúng chính tả. Chưa có phép đo để kết luận app nào nhận dạng tiếng Việt chính xác hơn, nhẹ hơn hoặc nhanh hơn.

## Đã áp dụng

- Tùy chọn LyricsPlus mặc định tắt. Cần bật cả tra cứu mạng và quyền LyricsPlus. Chỉ gửi metadata, không gửi âm thanh.
- Khi bật, tra LRCLIB và LyricsPlus song song với timeout và giới hạn phản hồi. Chọn bản có mốc từng chữ trước bản theo câu hoặc chữ thuần, sau khi kiểm tra khớp bài. Nguồn lỗi không ngăn nguồn còn lại.
- Ghép các mảnh âm tiết theo chữ trong câu gốc, không tự chèn khoảng trắng vào giữa một từ. Giữ thời lượng riêng của các chữ lặp. Nếu không ghép được, giữ lời và mốc câu, không gắn mốc từ giả.
- Nút **Tìm lời** lấy bản xem trước cho cả bài đã có cache. Chỉ **Lưu lời** mới chọn bản mới; không xóa cache/file gốc hoặc tự thay bản sửa tay. Bản nháp phải lưu trước khi tìm bản khác.
- Bù thời gian bằng `[offset:ms]`: số dương chạy lời sớm hơn, số âm chạy muộn hơn. Ô chỉnh giới hạn từ -10000 đến 10000 ms. Panel, Visualizer và các chỗ dùng parser chung nhận cùng độ bù.

Không áp dụng tự căn bằng fingerprint/FFT, dịch lời, module provider hoặc toàn bộ giao diện Android. Các phần đó không cần để thực hiện cải tiến này.

## Kiểm tra và giới hạn

- Máy chủ `lyricsplus.binimum.org` trả lời mẫu “Shape of You” với metadata và mốc từ. Bộ chuyển đổi Rust đã chuyển phản hồi đó thành LRC có mốc từng chữ. Mẫu tải để kiểm tra được dọn, không đưa toàn bộ lời vào bộ test.
- Truy vấn “Nơi Này Có Anh” / “Sơn Tùng M-TP” chưa tìm thấy lời trên máy chủ này. Không cam kết độ phủ bài Việt.
- Máy chủ `lyricsplus.prjktla.my.id` trả lỗi 502 trong lần kiểm tra, nên không thêm nó làm mirror mặc định.
- Không ghi đè cache cũ khi bật tùy chọn. Dùng **Tìm lời** rồi xem trước và lưu nếu muốn đổi.
- `npm run check`, các test lyrics/icons/themes, build web và toàn bộ 36 test Rust đều qua. Thử giao diện với IPC giả lập xác nhận: preview không lưu, không đổi bản nháp đang sửa, lưu offset dịch đúng mốc, quyền LyricsPlus không bật quyền upload cloud. Chưa build lại bộ cài desktop hoặc thử API bằng native IPC trong app thật.
- Tích hợp viết riêng theo giao thức phản hồi. Không chép mã Kotlin GPL-3.0 của Mochy vào dự án MIT. Quyền sử dụng nội dung lyrics và điều khoản dịch vụ vẫn thuộc từng bên.

## Mã nguồn tham khảo

- [Mochy README: Android, desktop và GPL-3.0](https://github.com/realidkroo/Mochy-Music/blob/eafbbe6d1c2a6590cbdfa045b42b572611bb2133/README.md)
- [Registry nguồn lyrics](https://github.com/realidkroo/Mochy-Music/blob/eafbbe6d1c2a6590cbdfa045b42b572611bb2133/app/src/main/kotlin/com/mochyy/mochymusic/lyrics/LyricsProviderRegistry.kt)
- [LyricsHelper: chấm điểm và chọn nguồn](https://github.com/realidkroo/Mochy-Music/blob/eafbbe6d1c2a6590cbdfa045b42b572611bb2133/app/src/main/kotlin/com/mochyy/mochymusic/lyrics/LyricsHelper.kt)
- [Công cụ offset của Mochy](https://github.com/realidkroo/Mochy-Music/blob/eafbbe6d1c2a6590cbdfa045b42b572611bb2133/app/src/main/kotlin/com/mochyy/mochymusic/ui/player/menu/LyricsOffsetSheet.kt)
- [Dự án LyricsPlus](https://github.com/ibratabian17/LyricsPlus)
