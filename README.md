# 🎵 Trantramy — Hướng Dẫn Sử Dụng

> **Phiên bản:** 0.1.8 · **Nền tảng:** Windows 10/11 (64-bit)

---

## 📖 Giới Thiệu

**Trantramy** là ứng dụng nghe nhạc & karaoke cá nhân được thiết kế dành riêng cho trải nghiệm nghe nhạc tại nhà theo phong cách phòng thu. Lấy cảm hứng từ đĩa than vinyl cổ điển, Trantramy kết hợp giữa thẩm mỹ hiện đại với những tính năng thực dụng:

- 🎙️ Phòng karaoke với lời bài hát đồng bộ từng từ theo nhịp nhạc
- 🌐 Tìm kiếm & stream trực tiếp từ **YouTube** và **SoundCloud**
- 📀 Đĩa than vinyl xoay với cần đĩa cơ học — kéo để phát/dừng
- 🎚️ Bộ chỉnh âm Equalizer 10 dải tần
- 🏝️ Dynamic Island hiển thị lời bài hát thu nhỏ trên thanh tiêu đề
- 🎨 Giao diện tùy biến sâu — màu sắc, nền, skin vinyl, font chữ

---

## 💻 Yêu Cầu Hệ Thống

| Thành phần | Yêu cầu tối thiểu |
|------------|-------------------|
| Hệ điều hành | Windows 10 (64-bit) trở lên |
| RAM | 2 GB |
| Dung lượng | 50 MB trống |
| Kết nối mạng | Cần để stream nhạc online (không bắt buộc cho nhạc local) |

---

## 📥 Cài Đặt

### Cách 1 — File `.exe` Portable (không cần cài)

1. Tải file **`Trantramy.exe`**
2. Đặt vào thư mục bất kỳ (ví dụ: `C:\Trantramy\`)
3. **Double-click** để chạy ngay — không cần cài đặt

> [!NOTE]
> Dữ liệu ứng dụng (thư viện, cài đặt) được lưu tự động tại:
> `C:\Users\<Tên bạn>\AppData\Roaming\com.trantramy.player\`

### Cách 2 — File Installer `dlc-setup.exe`

1. Tải file **`dlc-setup.exe`**
2. Chạy file → nhấn **Tiếp theo** → chọn thư mục cài đặt → **Cài đặt**
3. Shortcut sẽ xuất hiện trên Desktop và Start Menu
4. Nhấn **Mở Trantramy** để bắt đầu

> [!TIP]
> Nên dùng installer nếu muốn app tự cập nhật và tích hợp vào Start Menu.

---

## 🚀 Khởi Động Lần Đầu

Khi mở app lần đầu, bạn sẽ thấy **màn hình chính** gồm:

```
┌─────────────────────────────────────────────┐
│  🔊 Trantramy          [–] [□] [×]          │
├──────┬──────────────────────────────────────┤
│      │                                      │
│ Nav  │        Nội dung chính                │
│      │                                      │
├──────┴──────────────────────────────────────┤
│         🎵 Player Bar (thanh phát nhạc)     │
└─────────────────────────────────────────────┘
```

### Thêm nhạc vào thư viện

Có **3 cách** để thêm nhạc:

| Cách | Thao tác |
|------|----------|
| **Kéo thả** | Kéo file nhạc/thư mục từ File Explorer vào cửa sổ app |
| **Nút thêm thư mục** | Nhấn icon 📁 trên thanh điều hướng → chọn thư mục |
| **Dán link** | Dán URL YouTube/SoundCloud vào ô tìm kiếm → Enter |

---

## 🏠 Màn Hình Chính

### Thanh điều hướng (bên trái)

| Icon | Chức năng |
|------|-----------|
| 🏠 | **Trang chủ** — gợi ý bài hát, lịch sử nghe |
| 🎵 | **Thư viện** — danh sách nhạc local của bạn |
| 🌐 | **Trực tuyến** — tìm kiếm & stream YouTube/SoundCloud |
| ⚙️ | **Cài đặt** — tuỳ chỉnh giao diện và hành vi |

### Thanh phát nhạc (dưới cùng)

- **▶ / ⏸** — Phát / Tạm dừng
- **⏮ ⏭** — Bài trước / Bài kế tiếp
- **Thanh tiến trình** — Kéo để tua nhanh/chậm
- **🔊** — Điều chỉnh âm lượng
- **🎚️** — Mở Equalizer
- **📀** — Mở Phòng Karaoke

---

## 🎙️ Phòng Karaoke

Đây là tính năng **trọng tâm** của Trantramy. Nhấn icon 📀 trên thanh phát nhạc để vào.

### Giao diện phòng karaoke

```
┌──────────────────────────────────────────┐
│          [Lời dòng trước - mờ]           │
│                                          │
│    ═══ LỜI ĐANG HÁT — SÁNG RỰC ═══     │
│         (từng chữ sáng lên theo nhịp)    │
│                                          │
│          [Lời dòng kế - mờ]             │
├──────────────────────────────────────────┤
│  📀 Đĩa vinyl xoay   🎵 Controls        │
└──────────────────────────────────────────┘
```

### Đĩa than vinyl

| Thao tác | Kết quả |
|----------|---------|
| **Click vào đĩa** | Phát / Tạm dừng |
| **Kéo cần đĩa vào đĩa** | Phát nhạc |
| **Kéo cần đĩa ra khỏi đĩa** | Tạm dừng |
| **Đĩa quay** | Đang phát — ảnh bìa album phủ toàn mặt đĩa và quay cùng |

### Chế độ hiển thị lời

Nhấn icon **👁** để chuyển đổi:

- **Flow (mặc định)** — Hiện 3 dòng: trước / đang hát / kế tiếp
- **Vinyl** — Lời hiển thị kết hợp với đĩa vinyl
- **Stage** — Toàn màn hình, chữ lớn như sân khấu

### Gọi bài & hàng chờ

Nhấn **📋 Gọi bài** (góc phòng karaoke):
- **Tab Gọi bài**: Tìm kiếm theo tên bài / ca sĩ → Thêm vào hàng chờ
- **Tab Hàng chờ**: Xem và sắp xếp lại thứ tự bài sắp hát

### Tải lời bài hát

1. Trong phòng karaoke, nhấn **✏️ Sửa lời**
2. App tự động tìm lời đồng bộ (`.lrc`) từ cơ sở dữ liệu
3. Nếu chưa có: dán lời thủ công vào ô văn bản

---

## 🌐 Tìm Kiếm & Stream Trực Tuyến

### Tìm kiếm

1. Nhấn **🌐 Trực tuyến** trên thanh điều hướng
2. Nhập tên bài hát vào ô tìm kiếm
3. Chọn nguồn: **Tất cả / YouTube / SoundCloud / Local**
4. Nhấn kết quả để **nghe ngay** hoặc nhấn **+** để thêm vào hàng chờ

### Dán link trực tiếp

Hỗ trợ dán thẳng vào ô tìm kiếm:
- Link bài YouTube: `https://youtube.com/watch?v=...`
- Link playlist YouTube: `https://youtube.com/playlist?list=...`
- Link SoundCloud: `https://soundcloud.com/...`
- Link Spotify (sẽ tìm bản tương đương trên YouTube): `https://open.spotify.com/track/...`

### Tải xuống

Nhấn icon **⬇️** bên cạnh bài hát để tải về máy và thêm vào thư viện.

---

## 🎚️ Equalizer

1. Nhấn icon **🎚️** trên thanh phát nhạc
2. Kéo các thanh trượt để điều chỉnh 10 dải tần (60Hz → 16kHz)
3. Chọn **Preset** có sẵn: Pop / Rock / Classical / Bass Boost / v.v.
4. Nhấn **Lưu** để giữ cài đặt

---

## 🏝️ Dynamic Island

Dynamic Island là thanh lời bài hát thu nhỏ hiển thị trên **tiêu đề cửa sổ** khi bạn thu nhỏ phòng karaoke:

- Lời bài hát cuộn liên tục ngay trên thanh tiêu đề
- Nhấn vào để quay lại phòng karaoke đầy đủ
- Bật/tắt trong **⚙️ Cài đặt → Dynamic Island**

---

## 🎨 Tuỳ Chỉnh Giao Diện

Vào **⚙️ Cài đặt → Giao diện** để thay đổi:

### Chủ đề & Màu sắc

| Tuỳ chọn | Mô tả |
|----------|-------|
| **Màu chủ đạo** | Màu accent của toàn bộ giao diện |
| **Nền** | Apple Glass / Glow / OLED / Hình nền tuỳ chọn |
| **Hình nền** | Chọn ảnh từ máy tính |

### Vinyl & Phòng Karaoke

| Tuỳ chọn | Mô tả |
|----------|-------|
| **Skin đĩa** | Marble (mặc định) / Album / Black / Cyberpunk / Gold |
| **Vị trí ảnh bìa** | Toàn mặt / Trung tâm / Viền |
| **Màu đĩa** | Chọn màu tùy thích qua color picker |
| **Tốc độ quay** | 33 RPM (mặc định) / 45 RPM / 78 RPM |
| **Hiện cần đĩa** | Bật/tắt tonearm bên phải đĩa |
| **Font chữ lời** | Kích thước chữ nhỏ / vừa / lớn |

### Nhập Danh Sách Spotify

1. Vào **⚙️ Cài đặt → Nhập Spotify**
2. Dán link playlist Spotify
3. App tự tìm và thêm các bài tương đương từ YouTube

---

## ⌨️ Phím Tắt

| Phím | Chức năng |
|------|-----------|
| `Space` | Phát / Tạm dừng |
| `←` / `→` | Tua lùi / Tua tới 5 giây |
| `↑` / `↓` | Tăng / Giảm âm lượng |
| `Ctrl + K` | Mở phòng karaoke |
| `Esc` | Đóng panel / Phòng karaoke |
| `F11` | Toàn màn hình |

---

## ❓ Câu Hỏi Thường Gặp

**Q: App có cần internet không?**
> Chỉ cần internet khi stream nhạc YouTube/SoundCloud. Nhạc local nghe hoàn toàn offline.

**Q: Lời bài hát không đồng bộ đúng?**
> Vào phòng karaoke → nhấn ✏️ Sửa lời → chỉnh offset thời gian hoặc chọn nguồn lời khác.

**Q: App có lưu danh sách phát không?**
> Có. Tạo playlist trong tab Thư viện → kéo thả bài hát vào.

**Q: Làm sao xóa bài khỏi thư viện?**
> Click chuột phải vào bài hát → **Xóa khỏi thư viện** (không xóa file gốc).

**Q: Dữ liệu được lưu ở đâu?**
> `%APPDATA%\com.trantramy.player\` — sao lưu thư mục này để không mất dữ liệu khi cài lại.

---

## 📞 Liên Hệ & Hỗ Trợ

Ứng dụng được phát triển bởi **Tran Tra My**.
Mọi góp ý, báo lỗi xin gửi về qua kênh liên lạc cá nhân.

---

*Trantramy v0.1.8 · © 2026 Tran Tra My · Mọi quyền được bảo lưu*
