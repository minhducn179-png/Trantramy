# Tài liệu dự án

- [SOURCE_REVIEW.md](SOURCE_REVIEW.md): kết quả rà soát giao diện WebView trước khi chuyển sang native và các giới hạn lúc kiểm tra.
- [RUST_MIGRATION.md](RUST_MIGRATION.md): kiến trúc Rust native đã triển khai trong `src-native/`.
- [Hướng dẫn native](../src-native/README.md): chạy, kiểm tra và đóng gói bản 2.0.

## Kiểm tra Rust native

```sh
cargo fmt --manifest-path src-native/Cargo.toml --check
cargo clippy --manifest-path src-native/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path src-native/Cargo.toml --all-targets --locked
```

Trên Windows có thể kiểm tra ZIP sau giải nén bằng `powershell -File src-native/verify-windows-package.ps1`. Lệnh này kiểm checksum, phiên bản, sidecar và mở cửa sổ với dữ liệu tạm. Thêm `-SkipGui` khi chạy trên CI.

## Kiểm tra mã WebView cũ

Dùng Node.js và Rust stable để kiểm tra phần mã cũ được giữ lại.

```sh
npm ci
npm run check
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

Kiểm tra giao diện dùng agent-browser đã ghim phiên bản trong devDependencies. Cài trình duyệt khi máy chưa có, rồi chạy các kịch bản tuần tự trên bản build production:

```sh
npx agent-browser install
npm run test:ui
```

Bộ UI kiểm tra thông tin bài hát, lời bài hát, cài đặt, bố cục và karaoke bằng IPC giả lập. Audio trong fixture được giữ ở thời điểm do test điều khiển. Các bài này không xác nhận chất lượng âm thanh, dịch vụ cloud hoặc stream thật.

Các tài liệu sản phẩm nội bộ và bản dựng Windows cục bộ vẫn được Git bỏ qua.
