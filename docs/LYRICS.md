# Lời bài hát tự động

## Quy trình viết lyric nhiều bước

Mọi model AI khi viết lời đều tự động đi qua quy trình tiêu chuẩn và đối chiếu theo `LYRIC-STANDARDS.md`. Cài đặt được tinh giản để người dùng chỉ cần chọn model AI viết lời và các tùy chọn cơ bản; hệ thống luôn tự động kích hoạt các bước kiểm duyệt chất lượng cao:

Quy trình cho bài chưa có lời hoặc **Viết lại bằng AI**:

1. **Chép phụ đề**: AI nhận diện đã chọn nghe bản thu và tạo bản lời thô. Chưa xuất/cache lyric ở bước này.
2. **Nghe đối chiếu**: Gemini nhận lại âm thanh cùng bản lời thô toàn bài. Model được yêu cầu đối chiếu phát âm, chính tả, ngữ cảnh, nhịp câu, hơi thở và chỗ ngắt. Chỉ sửa theo âm thanh, không sáng tác, dịch hoặc tự bỏ câu/chữ lặp.
3. **Phân câu**: app ghép chữ theo các vị trí kết thúc cụm hát mà Gemini phân tích. Không chia mặc định mỗi 10 chữ. Ranh giới đoạn upload không tự trở thành ranh giới câu.
4. **Nghe căn chữ**: nhận diện lại bằng Whisper, dùng lời đã sửa làm gợi ý. Lấy thời điểm bắt đầu và kết thúc riêng của từng chữ. Nếu AI chính là Gemini, bước này dùng Whisper offline đã cài; Gemini không cung cấp mốc từng chữ làm kết quả cuối của quy trình này.
5. **Kiểm tra rồi xuất**: toàn bộ chữ phải khớp theo thứ tự với lượt nhận diện cuối, có đủ mốc bắt đầu/kết thúc, không chồng/vượt bài. So khớp bỏ hoa/thường, dấu câu và dấu tiếng Việt; chữ xuất ra giữ bản đã rà. Chữ lặp phải khớp đủ từng lần, không chọn đoán một lần xuất hiện. Không chia đều thời gian khi thiếu mốc.

### Lượt viết lại ngữ cảnh bổ sung và bản nháp chưa rõ

Quyền **Thêm lượt nghe–viết lại ngữ cảnh, dấu câu và viết hoa bằng Gemini** mặc định tắt, kể cả khi nâng cấp. Nó cần quy trình nhiều bước được bật. Khi bật riêng quyền này, app thêm một lượt nghe giữa bước đối chiếu và phân câu, thành sáu giai đoạn. Gemini nhận lại âm thanh, bản chép thô và phân tích trước để kiểm tra câu nhận sai ngữ cảnh, viết hoa và dấu câu. Đây là một lượt gửi âm thanh có phí bổ sung, không phải chỉ rà văn bản.

`LYRIC-STANDARDS.md` là nguồn tiêu chuẩn chung, được nhúng trực tiếp vào prompt nghe–đối chiếu, nghe–viết lại và rà văn bản. Cùng nội dung được hiển thị tại **Tiêu chuẩn AI tự đối chiếu** trong cài đặt. Các quy tắc yêu cầu ưu tiên âm thanh, giữ cách diễn đạt/tên riêng/phương ngữ/chữ lặp, kiểm tra chính tả/viết hoa/dấu câu và không điền từ theo suy đoán. Quy tắc không bảo đảm model làm đúng; vẫn cần nghe kiểm tra.

Nếu còn giọng hát chưa xác định, Gemini trả vị trí và khoảng âm thanh cần nghe lại, không giữ từ đoán trong danh sách chữ đã xác nhận. App chèn **⟦không nghe rõ⟧** vào bản nháp, hiển thị khoảng tìm nghe lại là ước lượng, rồi dừng trước bước căn chữ. Không chia thời gian hay tạo mốc cho từ chưa rõ.

- Bản nháp tự động được cache riêng ở `.draft.json`, có cờ `draft_only`; không thay lời chỉnh sửa, `.lrc`, cache hoàn chỉnh hoặc cache cũ. Mở lại bài dùng bản nháp này, không tự gửi thêm AI có phí. Hàng đợi ghi nhận bản nháp cần nghe và sửa, không coi đó là lyric hoàn chỉnh.
- **Viết lại bằng AI** vẫn chỉ tạo bản xem trước. Chọn **Chỉnh sửa bản nháp** để sửa; bản cũ chưa thay đổi. Bản xem trước chưa chọn/lưu không tồn tại qua lần đóng app.
- Khi còn ký hiệu chưa rõ, app chặn **AI rà soát**, **Tạo mốc ước lượng** và **Lưu lời**. Máy chủ cũng từ chối lưu hoặc rà văn bản có ký hiệu này. Nghe lại và điền chữ đã xác nhận, hoặc chủ động yêu cầu lượt viết lại khác.
- Sân khấu karaoke ghi rõ bản nháp chưa căn mốc. Dynamic Island không lấy bản nháp chưa rõ làm câu hát đã hoàn tất.

Cần khóa Gemini, kể cả khi AI nhận diện chính là offline/Groq/OpenAI. Groq/OpenAI dùng lại nhà cung cấp đã chọn ở bước căn chữ. Offline và Gemini cần model Whisper offline đã cài. Mục rà soát bên dưới dùng chung khóa/model Gemini; quyền gửi âm thanh của quy trình nhiều bước không tự bật quyền gửi âm thanh cho AI chính hoặc quyền chỉ rà văn bản.

Mỗi lượt gửi âm thanh đều dùng ngân sách cloud đã lưu, kể cả gửi lỗi. Với Groq/OpenAI, cả chép lời, nghe đối chiếu Gemini và nghe căn chữ đều có thể gửi âm thanh. Với offline, lượt đối chiếu Gemini gửi âm thanh; nếu bật quyền viết lại bổ sung thì có thêm lượt Gemini này. Với Groq/OpenAI, quy trình có tối đa ba lượt âm thanh toàn bài, hoặc bốn khi bật lượt viết lại bổ sung; mỗi lượt có thể chia thành nhiều đoạn. Thay quyền/model hoặc tạm dừng hàng đợi sẽ được kiểm tra giữa các bước và ngay trước gửi. Yêu cầu đã gửi không thể thu hồi; không tự thử lại yêu cầu có phí.

Không ghi đè lời/file LRC đã có. **Tìm lời** thủ công vẫn chỉ tra cứu, không gửi âm thanh. Khi bật quy trình nhiều bước, việc tạo lời mới bắt đầu từ bản thu, không lấy lời tra cứu làm kết quả cuối để bỏ qua phân tích. **Viết lại bằng AI** vẫn chỉ trả bản xem trước; chọn và **Lưu lời** mới thay bản cũ. Bước lỗi/cạn ngân sách/thiếu mốc sẽ dừng mà không xuất lyric hoàn chỉnh. Kết quả chưa nghe rõ chỉ được giữ như bản nháp có đánh dấu nêu trên.

### Giới hạn của quy trình

- Đây là nhiều lượt nhận diện và đối chiếu, **không phải mô hình forced alignment chuyên dụng**. Whisper chỉ nhận gợi ý văn bản trong giới hạn ngữ cảnh của engine/nhà cung cấp. Gợi ý không bảo đảm nhận đúng; app vẫn so toàn bộ bản lời đã sửa với mốc nhận diện cuối.
- Gemini phân tích âm thanh theo đoạn tối đa 120 giây, có bản lời thô toàn bài để tham chiếu. Chữ bị cắt ở biên đoạn, giọng chồng hoặc chữ nhận diện khác bản rà có thể làm bước kiểm tra từ chối kết quả. Không tự ghép mốc cho trường hợp chưa rõ.
- Tối đa 2.000 chữ và 24.000 byte chữ phân tích mỗi bài. Kết quả sai cấu trúc, vị trí phân câu đảo/trùng hoặc phản hồi bị cắt đều bị từ chối.
- Mốc lấy từ Whisper thay vì nội suy đều, nhưng vẫn có thể lệch giọng hát. Nhãn **Cần kiểm tra** được giữ. Chưa đánh giá độ chính xác trên tập bài hát thật hoặc gọi Gemini thật trong đợt thay đổi này.

## Sử dụng

1. Mở **Cài đặt → Lời bài hát & AI**.
2. Chọn **Offline**, tải model Base hoặc Small, rồi lưu cấu hình. Công cụ whisper.cpp Windows x64 CPU được cài cùng lượt tải model.
3. Hoặc chọn Groq, Gemini hay OpenAI. Lưu API key và bật quyền gửi âm thanh trước khi dùng cloud.
4. Mở một bài hát. App thử cache/file LRC, nguồn lời có sẵn, phụ đề YouTube và AI đã cấu hình. Nguồn lời có sẵn gồm LRCLIB và LyricsPlus nếu bạn bật quyền riêng cho LyricsPlus; kết quả khớp tên bài/nghệ sĩ/thời lượng được ưu tiên mốc từ → mốc câu → chữ thuần.
5. Trong thư viện, chọn **Tạo lời tự động cho cả thư viện**. Kiểm tra xác nhận trước khi bắt đầu. Hàng đợi lưu tiến độ, có thể tạm dừng/tiếp tục và không tự chạy lại sau khi khởi động app.
6. Trong panel lời, chọn **Thiết lập AI → Rà soát chính tả và dấu câu**. Lưu khóa Gemini và cấp quyền gửi lời riêng. Bật **Tự rà soát sau khi tìm hoặc nhận diện lời mới** nếu muốn chạy bước này tự động. Mặc định hai quyền gửi âm thanh và gửi lời đều tắt. Lời đã lưu không bị tự rà hoặc thay thế.
7. Với lời có sẵn, mở nút bút hoặc **Rà soát lời có sẵn**, rồi bấm **AI rà soát**. Mặc định chỉ gửi chữ đến Gemini, không tải/nhận diện lại âm thanh. Xem các dòng trước/sau, chọn **Áp dụng bản nháp**, nghe kiểm tra rồi **Lưu lời**. Có thể dán chữ thuần vào editor. Với lời đã có mốc, cần giữ số dòng và số chữ mỗi dòng để giữ mốc cũ.
8. Bật **Căn lại theo giọng hát** khi cần nhận diện lại âm thanh. App dùng AI nhận diện đã cấu hình, đối chiếu từng chữ rồi tạo bản nháp có mốc. Bước này cần mốc từng chữ từ Whisper offline, Groq hoặc OpenAI, không dùng mốc ước lượng của Gemini. Chưa đối chiếu đủ chữ hoặc còn nhập nhằng do chữ lặp thì app từ chối thay mốc. Kết quả sửa chữ vẫn có thể xem và áp dụng riêng.

Trong panel lời có **Xóa lời đã lưu** và **Viết lại bằng AI**:

- **Xóa lời đã lưu → Xác nhận xóa lời**: xóa bản chỉnh sửa, cache hiện tại và bản nháp của bài trong app. File `.lrc` gốc không bị xóa. App ghi trạng thái đã xóa để không tự lấy lại từ file gốc, cache cũ, mạng hoặc hàng đợi. Trạng thái gắn với cùng định danh file/metadata như cache.
- **Viết lại bằng AI → Bắt đầu viết lại**: nhận diện lại trực tiếp từ âm thanh, bỏ qua lời cũ và nguồn tra cứu. Dùng cấu hình AI, quyền cloud và ngân sách hiện tại. Nếu đã bật quy trình nhiều bước, app chạy chép lời → nghe đối chiếu → lượt viết lại bổ sung nếu được cho phép → phân câu → nghe căn chữ → kiểm tra. Nếu còn chỗ chưa nghe rõ, app chỉ trả bản nháp có đánh dấu và không căn mốc. Nếu không, tùy chọn tự rà soát chữ hoạt động như trước. Đây không phải sáng tác lời mới.
- Kết quả viết lại chỉ là bản xem trước. **Giữ bản cũ** bỏ kết quả. **Dùng bản viết lại** thay bản nháp, rồi **Lưu lời** mới thay bản lưu. Lỗi nhận diện không xóa lời cũ. Bản viết lại có thể đổi số dòng/chữ mà không bị ép vào mốc cũ, kể cả khi đổi bài rồi quay lại bản nháp.
- Sau khi xóa, có thể chọn **Tôi đã có lời** để viết/dán lại và lưu. Bản lưu mới sẽ được ưu tiên. **Tạo lời tự động** sau khi xóa cũng chỉ tạo bản nháp AI, không tự khôi phục lời cũ. Tác vụ lưu/xóa/tạo lời đang chạy được tuần tự hóa; hãy chờ tác vụ hiện tại hoặc tạm dừng hàng đợi trước khi thử lại.

Với bài tiếng Việt, chọn **Ngôn ngữ gốc → Tiếng Việt**. Có thể tải và chọn Small thay Base; chưa đo mức cải thiện trên tập bài hát. Lời/cache cũ không tự bị ghi đè, nên mốc từng chữ mới chỉ áp dụng cho kết quả tạo mới.

Muốn thử nguồn mới, bật **Cài đặt → Lời bài hát → Cho phép tra cứu thêm LyricsPlus**, đồng thời bật tra cứu mạng, rồi lưu cấu hình. Quyền mới mặc định tắt, kể cả khi nâng từ cấu hình cũ. App gửi tên bài, nghệ sĩ, album và thời lượng đến `lyricsplus.binimum.org`, không gửi âm thanh. Nguồn có thể thiếu bài hoặc gián đoạn.

Với bài đã có lời, bấm **Tìm lời** để tra lại nguồn có sẵn mà không dùng AI. Đây là bản xem trước; **Lưu lời** mới chọn bản mới. Lưu bản nháp đang sửa trước khi tìm. File `.lrc` gốc và bản trong cache không bị tự xóa.

Trong editor có **Bù thời gian (ms)** cho lời có timestamp. Số dương chạy lời sớm hơn, số âm chạy muộn hơn. Bấm **Lưu lời** để áp dụng. Độ bù được lưu bằng tag LRC `[offset:ms]`, dùng chung cho panel, Visualizer và các chỗ đọc qua parser. Nó chỉ dịch toàn bài, không sửa được bản thu khác tốc độ hoặc mốc từng chữ sai.

Mặc định dùng offline, không cho phép upload cloud. Tra cứu LRCLIB/YouTube qua mạng được bật mặc định; LyricsPlus cần bật thêm quyền riêng. Tắt quyền tra cứu/tải mạng để không gửi metadata hoặc tải âm thanh YouTube. Quyền upload cloud là một cài đặt riêng.

Giới hạn mặc định: 600 giây mỗi bài, 1.800 giây âm thanh cloud mỗi ngày UTC và 50 bài mỗi lượt chạy. Nếu còn bài chờ sau giới hạn lượt, bấm **Tiếp tục**. Ngân sách này không phải giới hạn tiền trên tài khoản nhà cung cấp.

## Rà soát và căn lời

Đây là nhận diện lời từ bản thu, không phải sáng tác lời mới. Gemini chỉ được yêu cầu sửa lỗi rõ ràng, giữ ngôn ngữ, tên riêng, cách diễn đạt và chữ lặp. AI vẫn có thể sửa sai nghĩa; cần xem thay đổi và nghe lại.

Bộ kiểm tra trên máy chủ từ chối kết quả đổi số dòng, đổi số chữ từng dòng hoặc chèn timestamp. Các tag thời gian, khoảng nghỉ, metadata và độ bù trong bản gốc được giữ nguyên từng byte khi rà chữ. Giới hạn mỗi lượt: 24.000 byte và 2.000 chữ. Rà chữ dùng Gemini dù AI nhận diện được chọn là offline/Groq/OpenAI. Chi phí văn bản không thuộc ngân sách giây âm thanh cloud.

Căn lời đối chiếu toàn bộ chữ của bản tham chiếu theo thứ tự, bỏ khác biệt dấu tiếng Việt, hoa/thường và dấu câu khi so khớp. Phải có ít nhất 85% chữ nhận diện được dùng, mọi chữ tham chiếu đều khớp và không có nhiều cách chọn lần xuất hiện. Không chia đều thời gian cho chữ không khớp. Mốc mới thay độ bù cũ bằng thời gian tuyệt đối của bản nhận diện. Đây không phải mô hình forced alignment chuyên dụng; nhận diện sai hoặc lời khác bản thu có thể làm căn bị từ chối. Mốc nhận diện vẫn có thể lệch giọng hát.

Lời có mốc dùng luồng rà chữ để bỏ qua xử lý âm thanh. Chưa có benchmark thời gian trên bài hát thật. Khi tự rà thất bại, app giữ kết quả tìm/nhận diện ban đầu và hiện lý do, không tự gửi lại có tính phí.

## Bảo vệ dữ liệu

- Bản chỉnh sửa trong app được ưu tiên. Tự động tạo lời không ghi đè file `.lrc` gốc hoặc lời đã có.
- Cache gắn với đường dẫn, metadata, dung lượng và thời điểm sửa file. File đổi trong lúc nhận diện sẽ không nhận kết quả cũ.
- Cache ghi qua file tạm. Kết quả quá lớn bị từ chối trước khi thay bản cũ.
- Nội dung đang sửa và bản xem trước được giữ theo bài trong phiên làm việc. Kết quả AI đến muộn không thay nội dung đang sửa. Các bản chưa lưu này không tồn tại qua lần đóng app. Riêng bản nháp tự động có vùng chưa rõ được cache tách biệt để mở lại không tự gửi AI lần nữa.
- API key mới nằm trong Windows Credential Manager, không lưu vào cấu hình JSON hay localStorage. App không hiển thị lại khóa đã lưu.
- Bản cũ có thể còn API key dạng rõ trong localStorage. Mở cài đặt và dùng nút chuyển khóa cũ sang kho Windows. Khóa chỉ bị gỡ khỏi localStorage sau khi chuyển thành công.
- Endpoint STT tùy chỉnh của bản cũ không được tự chuyển vào pipeline mới. Khóa thuộc endpoint này được giữ nguyên, không tự gửi đến nhà cung cấp khác. Ghi lại khóa nếu còn cần, rồi gỡ bản rõ trong localStorage.
- Trong chế độ thường, chỉ nhà cung cấp đã chọn nhận âm thanh khi có quyền cloud. Quy trình nhiều bước chỉ gửi thêm âm thanh/phụ đề đến Gemini khi đã bật quyền riêng nêu trên. App kiểm tra lại quyền trước các lần gửi. Thay đổi quyền không thu hồi được yêu cầu đã gửi.
- Ngân sách cloud được ghi trước khi gửi, gồm cả lượt gửi lỗi. Lỗi xác thực, quota hoặc ngân sách làm hàng đợi tạm dừng. Ngân sách hỏng chặn cloud thay vì cấp lại lượt dùng.
- Tiến trình whisper.cpp/yt-dlp của pipeline thuộc Windows Job Object. Đóng handle của app sẽ dừng nhóm tiến trình con, kể cả khi app bị buộc tắt.
- Model và gói runtime được ghim phiên bản/checksum. Chỉ cài sau khi SHA-256 khớp. Runtime được kiểm tra khởi động khi cài.
- Giải mã bằng Symphonia và resample bằng Rubato sang WAV mono 16 kHz. Không cần ffmpeg.

## Giới hạn

- AI có thể sai chữ, bỏ sót lời hoặc nhận nhầm nhạc nền. Cần kiểm tra các kết quả mang nhãn **Cần kiểm tra**.
- Chưa đánh giá độ chính xác trên tập bài hát. Không cam kết mọi bài đều có lời hay thời gian xử lý cố định.
- Whisper offline yêu cầu whisper.cpp tách ở ranh giới từ (`-ml 1 -sow`), rồi giữ mốc bắt đầu/kết thúc từng từ trong LRC. Không chia đều cả câu để thay thế mốc này. Chữ lặp được giữ theo từng lần xuất hiện, chữ hát nhanh/chậm có thời lượng riêng và khoảng nghỉ không bị kéo thành thời lượng hát. Mốc của Whisper vẫn có thể lệch giọng hát; phiên thay đổi này chỉ kiểm thử bằng dữ liệu mẫu, chưa chạy model trên bài hát thật.
- Phụ đề JSON3 dùng `tOffsetMs` nếu có đủ mốc từng chữ. Câu lặp có mốc bắt đầu mới không bị xóa chỉ vì giống chữ, kể cả cửa sổ hiển thị chồng nhau. Cửa sổ phụ đề cuộn có nội dung tăng dần vẫn dùng so khớp phần chữ trùng; lời lặp một phần có thể còn nhập nhằng nếu nguồn thiếu mốc.
- Hiệu ứng từng chữ từ lời chỉ có mốc câu vẫn là ước lượng, không thể xác định tốc độ hát từ văn bản đơn thuần. Gemini cũng được đánh dấu thời gian ước lượng.
- LRCLIB yêu cầu khớp tên bài/nghệ sĩ và thời lượng. Metadata thiếu hoặc khác phiên bản có thể bị bỏ qua để tránh chọn nhầm.
- Chỉ nhận phụ đề có thể xác định ngôn ngữ gốc, không lấy bản dịch tự động.
- Stream không phải YouTube cần tải về máy trước khi dùng STT. Không hỗ trợ mọi codec, DRM, video cần đăng nhập hoặc giới hạn vùng.
- Phiên này đã thử tải Base/runtime, phiên âm giọng đọc tổng hợp và luồng YouTube → tải âm thanh → STT offline trong app thật. Có sai từ trong mẫu YouTube. Chưa thử cloud bằng API key thật, model Small hoặc luồng lấy phụ đề thật. Tài nguyên tải riêng cho kiểm thử được dọn sau khi thử; hãy tải model trong cài đặt trước khi dùng offline.
- Thư viện đang chạy được xử lý tuần tự để hạn chế tài nguyên. Tạm dừng trước khi thoát nếu đang nhận diện. Một upload đang gửi không thể thu hồi.

## Kiểm thử

```sh
npm run check
npm run test:lyrics
npm run test:lyrics-ui
npm run test:icons
npm run test:themes
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri -- build --ci
```

`test:lyrics` kiểm tra parser, chữ lặp, chữ nhanh/chậm, khoảng nghỉ, sửa chính tả không đổi mốc, độ bù LRC, căn theo transcript và việc giữ tiến độ mới khi IPC trả snapshot cũ. `test:lyrics-ui` dùng IPC giả lập, kiểm tra bản xem trước, áp dụng trước khi lưu, nhánh chỉ rà chữ không nhận diện âm thanh, căn mốc, lỗi quyền, phản hồi đến muộn và giao diện hẹp. Chạy các bộ kiểm thử giao diện lần lượt, không mở đồng thời nhiều Vite/SvelteKit trên cùng thư mục. Test Rust kiểm tra metadata/ngôn ngữ LyricsPlus, ghép mảnh âm tiết và phân loại chất lượng thời gian. Xem `MOCHY-COMPARISON.md` để biết phạm vi đối chiếu Mochy và giới hạn thử nguồn thật. Bộ kiểm thử Rust bao gồm bảo vệ bản gốc, cache, timestamp, phụ đề lặp, giải mã/resample, quyền cloud, ngân sách, lưu hàng đợi, kho khóa Windows và dừng tiến trình con bằng Job Object.

Đợt sửa mốc từng chữ đã chạy `check`, `test:lyrics`, `test:icons`, `test:themes`, build web và bộ kiểm thử Rust. Thử giao diện trong trình duyệt với IPC giả lập xác nhận: thêm chữ bị chặn trước khi lưu; sửa dấu giữ nguyên mốc; chữ nhanh/chậm và khoảng nghỉ có tiến độ riêng. Đây không phải kiểm thử STT bằng bài hát thật hoặc build lại bộ cài desktop. Trong lần thử trình duyệt, vùng `win-proximity-zone` có lúc che nút sửa ở đầu panel; mở bằng bàn phím vẫn được. Vấn đề lớp phủ này chưa sửa trong thay đổi lyrics.

Đợt nâng cấp luồng rà soát đã chạy `check` không có lỗi/cảnh báo, kiểm thử lyric/state/icon/theme, build web và 38 kiểm thử Rust. Kiểm thử giao diện giả lập đạt; bộ layout hiện có cũng đạt riêng ở 1440×900 và 520×400. Chưa gọi Gemini bằng khóa thật, chưa đánh giá nhận diện/căn lời bằng bài hát thật trong đợt này, chưa đo tốc độ và chưa build lại bộ cài desktop. `cargo fmt --check` toàn kho vẫn báo định dạng tại `lib.rs` và các module khác; không format lại toàn kho ngoài phạm vi.

Đợt thêm xóa/viết lại đã chạy `check`, kiểm thử lyric/state/icon, build web và 40 kiểm thử Rust. Kiểm thử giao diện IPC giả lập đạt các nhánh xác nhận/hủy, lỗi xóa, xóa đồng bộ bản nháp giữa hai panel, mở lại sau xóa, viết lại thất bại, xem trước không ghi đè, bỏ kết quả và lưu bản viết lại khác số dòng/chữ. Thử giao diện không dùng âm thanh hoặc khóa AI thật; chưa build lại bộ cài desktop.

Đợt quy trình nhiều bước (0.1.5) đã chạy `check` không có lỗi/cảnh báo, kiểm thử lyric/state/karaoke/icon/theme, 46 kiểm thử Rust, các kiểm thử giao diện lyrics/karaoke và layout ở 1440×900, 1100×720, 800×600, 520×400. Đã đóng gói EXE/MSI/NSIS và cập nhật bản cài đặt cùng portable sau sao lưu; các tệp dữ liệu đã sao lưu giữ nguyên khi cài. Kiểm thử mới dùng dữ liệu mẫu/IPC giả lập, không dùng khóa AI hoặc bài hát thật để đo chất lượng. Xem `.impeccable/review/lyrics-pipeline-review.md` để biết phạm vi.

Đợt nghe–viết lại ngữ cảnh (0.1.6) đã chạy `check` không có lỗi/cảnh báo, kiểm thử lyric/state/karaoke/icon/theme, 52 kiểm thử Rust, giao diện lyrics/karaoke giả lập IPC và layout ở 1440×900, 1100×720, 800×600, 520×400. Kiểm tra thêm gồm quyền riêng mặc định tắt, thứ tự sáu giai đoạn, dừng khi lỗi/hủy, vùng chưa rõ không đi vào căn mốc, cache nháp riêng, chặn lưu/rà chữ khi còn ký hiệu và giữ viết hoa/dấu câu khi căn chữ. Đã đóng gói EXE/MSI/NSIS và cập nhật bản cài đặt, portable cùng alias bộ cài sau sao lưu; 36 tệp dữ liệu/cài đặt và bản sao lưu có SHA-256 giữ nguyên. Đã đối chiếu byte của executable và payload frontend nhúng. Chưa thử chất lượng bằng bài hát/Gemini thật hoặc tương tác cửa sổ native của gói mới. Chi tiết: `.impeccable/review/lyrics-context-review.md`, `.pi/windows-verify-0.1.6.log`; có thể chạy lại kiểm tra đóng gói bằng `node .pi/verify-windows-0.1.6.mjs` từ thư mục dự án.

Đợt hoàn thiện kiểm thử native và đóng gói sửa lỗi Windows (0.1.7) đã sửa `src-tauri/src/main.rs` để giữ lại các cờ tham số `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` được kế thừa từ môi trường ngoài, cho phép kết nối CDP/agent-browser phục vụ kiểm thử thực tế. Đã chạy kiểm thử native smoke trên app thật chạy qua Tauri IPC (`http://tauri.localhost/`), xác nhận: quyền cloud mặc định tắt, 16 tiêu chuẩn tự đối chiếu đầy đủ, Fullscreen API hoạt động trên WebView2 native, chế độ ẩn/hiện điều khiển karaoke hoạt động đúng với bàn phím, tìm kiếm bài hát không khớp trả về an toàn, hàng chờ trống hoạt động đúng, đóng phòng bằng Escape và các tùy chọn phát nhạc được giữ nguyên. Toàn bộ 52 bài kiểm thử Rust, kiểm thử JS (`test:lyrics`, `test:karaoke`, `test:icons`, `test:themes`) và `svelte-check` đều đạt. Đã đóng gói EXE, MSI và NSIS, cập nhật an toàn bản cài đặt (`%LOCALAPPDATA%\dlc\dlc.exe`), bản portable (`C:\miudich\dlc.exe`) và bộ cài (`C:\miudich\dlc-setup.exe`) sau khi sao lưu an toàn 37 tệp dữ liệu/cài đặt (`dlc-before-0.1.7-20261115-211824`). Toàn bộ 37 tệp dữ liệu gốc và bản sao lưu giữ nguyên SHA-256. Kiểm tra đóng gói lặp lại qua `node .pi/verify-windows-0.1.7.mjs`. Giới hạn: kiểm thử smoke trên ứng dụng native không chạy nhạc thật, không gọi API Gemini/Groq/OpenAI thật có phí và chưa đánh giá độ chính xác lời trên bài hát thật. Chi tiết: `.impeccable/review/native-smoke-0.1.7.md`, `.pi/windows-verify-0.1.7.log`.

Detector giao diện cảnh báo gradient chữ trong `LyricsPanel.svelte`. Gradient này biểu diễn tiến độ karaoke, không phải trang trí. Cảnh báo được giữ nguyên, không tắt luật kiểm tra.
