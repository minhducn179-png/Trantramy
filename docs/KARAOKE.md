# Phòng karaoke

Mở **Phòng karaoke** ở thanh điều hướng hoặc nút micro karaoke trên thanh phát bên dưới. Karaoke thay cho mục **Trình diễn sóng nhạc (Visualizer)**; lối mở nhanh trong Cài đặt → Dynamic Island cũng mở phòng karaoke.

- **Gọi bài**: tìm trong thư viện, hoặc tìm trên YouTube bằng nguồn **Beat karaoke** / **Có giọng**. Nút **+** thêm bài vào lượt hát. Nếu chưa có bài đang phát, bài đầu tiên bắt đầu ngay.
- **Chờ hát**: xem các bài còn lại. Nút mũi tên đưa bài lên lượt tiếp theo; nút **×** bỏ bài khỏi hàng đợi. Cả hai không ngắt bài đang hát.
- **Bỏ qua bài**: chuyển sang lượt tiếp theo. Hết lượt thì dừng, không tự lặp hoặc thêm gợi ý trong lúc phòng đang mở.
- **Tìm bản beat / Tìm bản có giọng**: tìm nguồn thay thế cho bài đang hát. **Dùng bản này** thay bài hiện tại và phát từ đầu. Các bài chờ còn lại được giữ nguyên.
- **Sân khấu riêng** chiếm cửa sổ ứng dụng, không mở cửa sổ Windows thứ hai. Mặc định bảng gọi bài được thu lại trên cả desktop và màn hình nhỏ. Nút **Gọi bài** mở công cụ khi cần; **Về sân khấu** thu lại.
- **Ẩn điều khiển** chỉ giữ nền và lời hát. Nút **Điều khiển** ở góc dưới hiện lại công cụ; Esc vẫn đóng phòng. Ẩn/hiện không đổi bài, hàng chờ hoặc vị trí phát.
- Nền dùng hình nền ứng dụng đã chọn; nếu chưa có hình nền thì dùng ảnh bìa bài hát. Theme và thiết lập hình nền được giữ nguyên.
- **Toàn bộ lời** hiển thị danh sách lời như bài hát thường, tự cuộn đến câu đang hát và cho phép bấm câu để tua. **Lời từng câu** trở lại chữ karaoke lớn với câu tiếp theo.
- **Lời bài hát** mở chung công cụ tìm lời, tạo lời tự động, dán/sửa lời, rà soát, bù mốc và lưu như bài hát thường. Bản tìm được chỉ là xem trước cho đến khi lưu. Lưu lời cập nhật sân khấu của đúng bản thu đang chọn. **Thiết lập AI** dẫn đến cấu hình lời hiện có, không thay quyền riêng tư.
- Lời lớn dùng chung parser LRC và mốc từng chữ với ứng dụng. Mốc nội suy từ câu được ghi rõ là ước lượng. Lời không có timestamp được hiển thị để cuộn đọc, không giả lập đồng bộ. Lời của bản có giọng không tự áp vào beat khác vì mốc có thể lệch.
- Nút toàn màn hình dùng Fullscreen API. Nếu thiết bị từ chối, phòng vẫn chiếm toàn bộ cửa sổ ứng dụng và hiển thị thông báo.
- Đóng phòng không dừng bài đang phát. Chính sách phát thông thường (ngẫu nhiên, lặp, bài liên quan) được khôi phục mà không thay các tùy chọn đã lưu.

## Nút lời bài hát trên thanh phát

Ở trang thông tin bài hát, nút lời cuộn tới và đưa tiêu điểm vào phần lời đã có trong trang; không mở thêm khung lời bên cạnh. Hàng chờ đang mở được đóng để không che phần lời. Ở Trang chủ, Thư viện, Khám phá, Cài đặt và playlist, nút lời vẫn bật/tắt khung lời như trước.

## Phạm vi

Không có chỉnh tông/tốc độ, micro, echo hay thu âm. Không tách giọng bằng AI. Kết quả tìm beat/có giọng là nguồn do người dùng chọn; tên kết quả không bảo đảm đúng bản thu hoặc đã loại giọng. Khả năng phát và lời phụ thuộc nguồn.

## Kiểm thử

```sh
npm run check
npm run test:karaoke
npm run build
npm run test:layout
npm run test:lyrics-ui
npm run test:karaoke:ui
node .pi/native-smoke.mjs
```

Kiểm thử bao gồm ẩn/hiện công cụ, tiêu điểm bàn phím, hai kiểu xem lời, tìm/xem trước/lưu lời và cập nhật sân khấu, hình nền và theme sáng. Đã xác minh Fullscreen API và giao diện native trong ứng dụng Tauri Windows (bản 0.1.7) qua WebView2 với IPC thật (`http://tauri.localhost/`), bao gồm vào/thoát toàn màn hình, ẩn điều khiển với phím khôi phục, tìm kiếm thư viện an toàn và đóng phòng bằng Escape.

### Giới hạn kiểm thử thực tế

Kiểm thử native là kiểm thử smoke quy trình và giao diện thực tế, không chứng minh chất lượng beat, độ chính xác mốc lời hay khả năng phát trực tiếp của nguồn YouTube thật. Trong đợt kiểm thử native, chưa chọn bài hát phát thật, chưa gửi yêu cầu tạo/tách lời AI có tính phí và không lưu cấu hình mới. Nguồn beat/có giọng trên YouTube thật phụ thuộc tính khả dụng mạng và bản quyền của nguồn phát.
