# Tiêu chuẩn đối chiếu và viết lời từ bản thu

## Căn cứ

1. Ưu tiên giọng hát thực tế. Dùng bản chép thô và ngữ cảnh toàn bài để kiểm tra, không dùng chúng để lấp lời thiếu.
2. Chỉ viết lại chữ/câu khi âm thanh hỗ trợ cách sửa. Giữ ngôn ngữ gốc, tên riêng, phương ngữ và cách diễn đạt nghệ thuật.
3. Giữ đủ từng lần lặp, tiếng đệm và câu hát lại. Nhạc dạo không phải lời hát.

## Chưa nghe rõ

4. Với giọng bị che, hát chồng hoặc phát âm chưa xác định, ghi khoảng chưa rõ vào `uncertain`. Bỏ chữ chưa xác nhận khỏi `words`.
5. Mỗi khoảng có `after_word` là số chữ đã nghe rõ đứng trước nó, cùng `start`/`end` theo giây trong đoạn âm thanh hiện tại. Vị trí đầu đoạn là 0.
6. Chỉ bỏ dấu chưa rõ sau lượt nghe lại nếu thực sự nghe được chữ. Nếu vẫn mơ hồ, giữ dấu. App sẽ chèn `⟦không nghe rõ⟧`; model không tự chèn ký hiệu này vào chữ.

## Chính tả, viết hoa và dấu câu

7. Viết đúng dấu tiếng Việt. Viết hoa đầu câu và tên riêng đã xác định. Giữ chữ viết tắt phù hợp; tránh viết hoa toàn câu.
8. Đặt dấu chấm khi câu đã hoàn ý. Dùng dấu phẩy theo cấu trúc và nghĩa, không gắn dấu phẩy vào mọi chỗ lấy hơi.
9. Dùng dấu hỏi/cảm thán khi cấu trúc câu và cách hát hỗ trợ. Không tự đổi nghĩa khẳng định thành nghi vấn hoặc cảm thán.
10. Dấu câu gắn vào chữ liền trước/sau theo chính tả, không thành phần tử riêng trong `words`. Không có khoảng trắng trước dấu chấm/phẩy/hỏi/cảm thán.
11. Xuống dòng theo cụm hát, chỗ ngắt và mạch nghĩa. Một dòng mới không nhất thiết là câu mới. Không ép kết thúc câu ở ranh giới đoạn upload.
12. Chữ ngân dài vẫn là chữ đã hát. Không thêm chữ để lấp thời gian. Dấu ba chấm không thay cho khoảng chưa nghe rõ.

## Tự kiểm tra trước khi trả kết quả

13. So lại từng câu với âm thanh, bản chép và các câu trước/sau. Câu nghe lạ nhưng có căn cứ âm thanh được giữ, không viết cho trôi chảy bằng suy đoán.
14. Kiểm tra đủ chữ lặp, dấu tiếng Việt, viết hoa, dấu câu và vị trí phân câu. Giữ mọi khoảng chưa rõ chưa giải quyết được.
15. Chỉ trả chữ và phân tích theo cấu trúc yêu cầu. App lấy mốc từng chữ từ lượt nhận diện riêng, không nhận mốc do model tự chia đều.
16. Trong chế độ chỉ rà văn bản, chỉ sửa lỗi chữ/viết hoa/dấu câu rõ ràng. Giữ nguyên vùng chưa rõ, số dòng, số chữ và mốc; không tuyên bố đã đối chiếu âm thanh.
