# Rovune

**Giữ nhịp riêng.**

Đọc: **rô-vun**. Tên sáng tạo gợi liên tưởng đến *rove* và *tune*: tìm nhạc, lưu nhạc và nghe theo gu riêng. Đây không phải một nghĩa từ điển.

Bộ nhận diện được thiết kế cho app nghe nhạc desktop hiện tại. **Chưa đổi tên DLC, app ID, icon hoặc dữ liệu trong ứng dụng.**

## Xem nhanh

- `brandboard.png`: bảng nhận diện hoàn chỉnh.
- `brandboard.svg`: cùng bố cục, dạng vector, nhúng font và artwork để mở độc lập.
- `theme-compatibility.png` / `.svg`: logo trên cả 10 họ theme hiện có.
- `app-icon-512.png` và `app-icon-32.png`: icon desktop lớn và kiểm tra kích thước nhỏ.

## Ý tưởng: Open Loop

Logo là chữ **R mở vòng**, có một khoảng ngắt rõ giữa thân và nét cong. Nét chéo kết thúc tạo nhịp chuyển động. Hình vòng mở được dùng lại trong artwork, không biến thành một dãy sóng nhạc hoặc nút play thông thường.

Wordmark **ROVUNE** là lettering vector riêng, nét tròn đồng nhất. Các file logo, wordmark và lockup chỉ chứa đường vector; không cần cài font để hiển thị đúng.

## Màu thương hiệu

| Tên | HEX | Vai trò |
| --- | --- | --- |
| Ink | `#101010` | Nền tối, chữ trên màu sáng |
| Paper | `#F5F0E8` | Nền sáng, chữ đảo màu |
| Apricot | `#FFC799` | Màu nhận diện chính, điểm nhấn |
| Mint | `#99FFE4` | Màu phụ, dùng tiết chế |
| Slate | `#A0A0A0` | Chữ phụ trên Ink |

Dùng **Ink** làm chữ trên Apricot/Mint/Paper. Không dùng chữ trắng trên các nền pastel này. Không dùng Slate làm chữ nhỏ trên Paper.

`tokens.json` chứa các giá trị để tái sử dụng. Đây là màu nhận diện, **không thay thế các palette giao diện** Catppuccin, One Dark Pro, Tokyo Night, Dracula, Rosé Pine, Nord, Gruvbox, Ayu, Kanagawa và Vesper.

## Typography

- **Be Vietnam Pro 400/600:** nội dung, tiêu đề và giao diện. Có ký tự tiếng Việt.
- **JetBrains Mono 400:** thời lượng, định dạng và thông tin kỹ thuật.
- **Wordmark:** dùng file vector có sẵn, không gõ lại bằng font khác.

Font WOFF2 và giấy phép đi kèm trong `fonts/`. Nếu cần dùng trong phần mềm thiết kế yêu cầu TTF/OTF, lấy bản desktop từ nhà phát hành font; WOFF2 ở đây dành cho hiển thị web. Không gọi các file WOFF2 là font desktop.

## Sử dụng logo

| Tài sản | Dùng khi |
| --- | --- |
| `logo.svg` | Dấu R màu Ink trên nền sáng |
| `logo-inverse.svg` | Dấu R màu Paper trên nền tối |
| `wordmark.svg` / `wordmark-inverse.svg` | Tên thương hiệu độc lập |
| `lockup.svg` / `lockup-inverse.svg` | Logo và tên theo bố cục ngang |
| `app-icon.svg` | Icon có nền Ink và dấu R Apricot |

- Khoảng trống tối thiểu quanh dấu R: **một bề dày nét**. Không tính phần này vào nội dung lân cận.
- Dấu R độc lập: tối thiểu **24 px**. Icon có nền: tối thiểu **32 px**.
- Wordmark: tối thiểu **120 px** ngang. Lockup: tối thiểu **176 px** ngang.
- Giữ nguyên tỷ lệ và khoảng ngắt. Không kéo dãn, xoay hoặc thêm viền, bóng sáng, hiệu ứng giọt nước vào chính dấu R.
- Hiệu ứng kính có thể thuộc về khung giao diện, không trở thành một phần cố định của logo.
- Khi đổi theme: chỉ đổi màu dấu R theo accent hoặc màu chữ có đủ tương phản. Giữ nguyên hình học. Có thể dùng bản một màu trên mọi palette.
- UI vẫn dùng **Lucide** cho nút và điều hướng. Dấu R là tài sản thương hiệu, không thay thư viện icon chức năng.

## Giọng thương hiệu

Gọn, gần gũi, bình tĩnh. Nói rõ việc người dùng có thể làm, tránh phô trương kỹ thuật.

Mẫu copy đề xuất, chưa áp dụng vào app:

- Trạng thái trống: “Chọn một bài. Bắt nhịp riêng.”
- Nhập nhạc: “Thêm nhạc của bạn.”
- Theme: “Chọn sắc thái của bạn.”
- Lỗi: “Chưa phát được bài này. Thử lại hoặc chọn nguồn khác.”

## Artwork & phạm vi

`brand-art.png` là artwork trừu tượng được tạo bằng AI cho bộ nhận diện này. Prompt và nguồn nằm trong `artwork.provenance.json`. Đây không phải ảnh chụp sản phẩm hoặc bìa album có thật. Brandboard là tài liệu nhận diện, không phải giao diện đã triển khai. Mẫu “03:45 / FLAC / 48 kHz” minh họa typography, không phải kết quả đo âm thanh.

Tên Rovune chưa xuất hiện như một app nghe nhạc trong lượt tìm kiếm sơ bộ của phiên làm việc này. Điều đó **không xác nhận** khả dụng tên miền, tài khoản mạng xã hội hoặc quyền đăng ký nhãn hiệu. Cần tra cứu pháp lý trước khi phát hành thương mại.

## Tạo lại & kiểm tra

Từ thư mục dự án, dùng Node có hỗ trợ TypeScript type stripping:

```bash
node brandkit/rovune/build.mjs
node brandkit/rovune/check.mjs
```

Script tạo lại SVG, tokens và font từ các dependency đang có, không cài thêm package. PNG xem trước được xuất bằng Chromium qua `agent-browser`; tạo lại SVG không tự cập nhật PNG.
