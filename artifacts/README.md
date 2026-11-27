# Artifacts cục bộ

Thư mục này giữ các bản Windows portable và các file liên quan khi tổ chức lại workspace. Các file `.exe` vẫn bị Git bỏ qua.

- `windows/Trantramy.exe`: bản portable đã chuyển từ thư mục gốc.
- `windows/dlc-setup.exe`: bộ cài đã chuyển từ thư mục gốc.
- `windows/yt-dlp.exe`: hard link đến `src-tauri/binaries/yt-dlp-x86_64-pc-windows-msvc.exe`.
- `../Trantramy/Trantramy.exe`: bản khác, giữ nguyên vì Windows đang khóa file.
- `../Trantramy/yt-dlp.exe`: hard link phục vụ bản portable bị khóa.

Không xóa hoặc thay thế các file này khi chưa kiểm tra hash và đường dẫn sidecar.
