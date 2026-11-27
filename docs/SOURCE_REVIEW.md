# Rà soát mã nguồn Trantramy/DLC

## 1. Phạm vi

Mã nguồn hiện tại là Trantramy/DLC v0.1.9. Giao diện dùng Svelte 5, TypeScript, SvelteKit và Vite. Backend desktop dùng Tauri 2 và Rust.

Đợt rà soát này đã kiểm tra:

- cấu trúc thư mục và các import;
- luồng thư viện, trình phát, lời bài hát, karaoke và nguồn trực tuyến;
- cấu hình build, CI, test và Rust backend;
- tệp trùng theo SHA-256;
- các tài nguyên mẫu không còn được tham chiếu.

## 2. Cấu trúc hiện tại

```text
src/
├── lib/
│   ├── api.ts                 # kiểu dữ liệu và IPC frontend
│   ├── audio/                 # engine âm thanh, EQ, waveform
│   ├── components/            # thành phần dùng chung
│   ├── discovery/             # trang chủ, khám phá, gợi ý
│   ├── karaoke/               # phòng karaoke
│   ├── library/               # thư viện, chi tiết, Spotify import
│   ├── lyrics/                # LRC, state, UI và AI lyrics
│   ├── player/                # state và thanh phát
│   └── settings/              # theme, palette và cài đặt
├── routes/
│   ├── +page.svelte           # cửa sổ chính
│   └── island/+page.svelte    # Dynamic Island
└── app.html

src-tauri/src/
├── lib.rs                     # Tauri commands và backend orchestration
├── ytdlp.rs                   # yt-dlp integration
└── ...
```

`src/lib/api.ts` vẫn là entry point ổn định cho các hàm IPC và kiểu dữ liệu dùng chung. Không tạo thêm lớp barrel index khi chưa cần. Cách này giảm số tệp trung gian và giữ import rõ ràng.

## 3. Tính năng đã triển khai

### Thư viện và phát nhạc

- quét thư mục nhạc;
- đọc metadata và cover;
- danh sách yêu thích, playlist và lịch sử;
- phát, dừng, tua, âm lượng, shuffle, repeat và queue;
- giữ thông tin bài được chọn độc lập với bài đang phát;
- giới hạn thử lại khi stream lỗi để tránh lặp vô hạn.

### Nguồn trực tuyến

- tìm kiếm nguồn trực tuyến;
- lấy stream qua backend;
- tải xuống qua yt-dlp;
- nhập Spotify để lấy metadata rồi tìm nguồn phát tương ứng.

Spotify không cung cấp audio trực tiếp trong luồng hiện tại. Ứng dụng dùng metadata Spotify và phân giải nguồn phát khác.

### Lời bài hát

- đọc và ghi LRC;
- căn theo câu hoặc từng từ;
- offset thời gian;
- sửa nội dung và giữ timestamp khi có thể;
- hàng đợi tạo lời;
- pipeline offline/cloud với consent, giới hạn thời gian và cấu hình model;
- xem trước, duyệt và lưu kết quả.

### Karaoke

- Studio dạng dialog hoặc khung nhúng;
- gọi bài và xếp hàng FIFO;
- ưu tiên, xóa và thay nguồn;
- hiển thị lời theo thời gian;
- lựa chọn nguồn có giọng hoặc không lời;
- giao diện đĩa than và Dynamic Island.

Mã hiện tại chưa cho thấy vocal separation, thu âm hoặc chấm điểm giọng hát thật. Các lựa chọn beat/có giọng là lựa chọn nguồn tìm kiếm.

## 4. Giao diện

Giao diện chính có:

1. thanh điều hướng và tìm kiếm bên trái;
2. vùng nội dung cho Trang chủ, Thư viện, Khám phá, playlist và Cài đặt;
3. thanh phát cố định bên dưới;
4. Studio lời bài hát, drawer hàng chờ và hộp thoại EQ;
5. karaoke và Dynamic Island.

Cài đặt gồm theme sáng/tối, palette, hình nền, mật độ, bo góc, waveform, âm thanh, lời bài hát và Dynamic Island.

## 5. Điểm kỹ thuật cần lưu ý

- EQ hiện quy đổi các giá trị band thành thay đổi gain tổng. Nó chưa áp dụng filter độc lập cho từng dải tần.
- Waveform có phần dữ liệu seed để hiển thị ổn định. Một phần khác lấy phổ đầu ra hệ thống trên Windows.
- Backend Rust đã chứa nhiều logic nghiệp vụ. Mục tiêu chuyển sang Rust nên bắt đầu bằng việc tái sử dụng lõi này, không viết lại mù toàn bộ.
- `yt-dlp` và các công cụ nhận dạng âm thanh vẫn là chương trình ngoài. Chuyển application code sang Rust không tự động thay thế các chương trình đó.

## 6. Dọn dẹp đã thực hiện

### Tổ chức mã nguồn

Đã chuyển 24 tệp frontend từ `src/lib/` phẳng vào các nhóm `audio`, `components`, `discovery`, `karaoke`, `library`, `lyrics`, `player` và `settings`. Đã cập nhật import trong route, component và test. Các biểu tượng nhạc dạo dùng Lucide; SVG tay cần được đánh dấu là hình minh họa.

Đợt kiểm tra tiếp theo đã sửa:

- nút lời trên trang chi tiết cuộn tới nội dung và chuyển focus, đồng thời đóng hàng chờ;
- nút thu nhỏ Studio đổi sang chế độ nhúng, nút phóng to mở lại Studio;
- lối tắt Thiết lập AI mở đúng tab AI;
- lưu cấu hình không tự cấp quyền gửi âm thanh hoặc lời đến cloud;
- bản nháp chưa xác nhận luôn có cảnh báo và không hiển thị như lời đã căn mốc;
- số tiêu chuẩn hiển thị lấy từ danh sách thực tế;
- khung Studio chia chiều cao bằng grid ở cả dialog và chế độ nhúng, giữ vùng sửa lời cuộn được và dùng toàn chiều rộng cho khung gọi bài trên cửa sổ hẹp;
- test bố cục đo vị trí nút thực tế thay vì phụ thuộc cách trình duyệt biểu diễn từ khóa CSS;
- test UI dùng Studio hiện tại thay cho drawer cũ, chạy tuần tự trên bản build production với audio do fixture điều khiển.

`agent-browser` được ghim trong devDependencies. CI dùng `npm ci` và chạy test logic frontend.

### Tệp trùng

Theo manifest của phiên dọn workspace trước:

- không có nhóm tệp mã nguồn giống byte trong tập đã quét;
- bản `yt-dlp.exe` trùng ở thư mục gốc đã được xóa;
- các bản portable giữ sidecar cạnh executable bằng hard link;
- bản executable đang bị khóa ở thời điểm đó được giữ nguyên;
- schema/icon có vai trò nền tảng riêng không bị xóa chỉ vì giống nội dung.

Manifest kiểm tra nằm tại `.pi/source-review/cleanup-manifest.json`. Đây là bằng chứng cục bộ của phiên trước, không phải lần kiểm tra đóng gói mới. Thư mục `.pi/` vẫn bị Git bỏ qua.

### Tài nguyên mẫu

Đã xóa các tệp starter không được tham chiếu:

- `static/svelte.svg`;
- `static/tauri.svg`;
- `static/vite.svg`.

## 7. Kết quả kiểm tra

| Lệnh | Kết quả |
|---|---|
| `npm run check` | Đạt, 0 lỗi, 0 cảnh báo |
| `npm test` | Đạt: icon, lyrics, theme, player, karaoke |
| `npm run build` | Đạt |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | Đạt: 52 test |
| `npm run test:lyrics-ui` | Không đạt: sau khi cập nhật nút mở theo nhãn hiện tại, test vẫn tìm nút `Chỉnh sửa lời bài hát` ở màn hình không hiển thị drawer mong đợi |
| `npm run test:layout` | Không kết thúc trong 180 giây; chưa có kết quả pass/fail |
| `npm run test:track-info` | Không hoàn tất trong Windows browser harness |

Các test UI chưa được đưa vào `npm test` vì browser harness chưa ổn định và một số kịch bản kiểm tra UI cũ không khớp với luồng hiển thị hiện tại. Không đánh dấu chúng là đạt.

## 8. Việc chưa làm

- chưa commit;
- chưa rewrite sang Rust;
- chưa đo benchmark trước migration;
- chưa kiểm tra audio native, cloud, streaming thật trong môi trường sản xuất;
- chưa xác nhận bản Windows portable sau khi đóng gói lại.
