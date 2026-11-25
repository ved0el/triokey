# Bài giới thiệu (tiếng Việt)

## Câu giới thiệu ngắn

1. Gõ Anh, Việt, Nhật trên một bàn phím, không cần bật/tắt bộ gõ tiếng Việt.
2. Bộ gõ tiếng Việt tự nhường khi IME CJK đang bật.
3. Gõ `windows` ra `windows`, gõ `tieesng` ra `tiếng`. Không cần chuyển chế độ.

## Bài đăng

Ngày nào mình cũng gõ Anh, Việt, Nhật trên Windows, và mình chán cảnh bật/tắt. Đang gõ Việt muốn sang Nhật thì phải bật hiragana, rồi còn phải tắt chế độ tiếng Việt; lúc quay lại thì làm ngược cả hai.

Nên mình viết TrioKey, một bộ gõ tiếng Việt (Telex) mã nguồn mở bằng Rust.

- Nó đọc xem IME CJK có thực sự đang mở trong cửa sổ hiện tại không rồi tự nhường. IME đóng thì nó gõ tiếng Việt, kể cả khi đang ở layout Nhật.
- Không có chế độ Việt/Anh. Mỗi từ được xét riêng: là âm tiết tiếng Việt hợp lệ thì thành tiếng Việt, còn lại giữ như đã gõ. Từ có hai cách đọc thì một lần Backspace là đổi sang cách kia.
- Nó không thêm layout bàn phím và mặc định không đăng ký gì với Windows. Chỉ là một ứng dụng ở khay với một file cấu hình dạng văn bản, không lưu lại phím bạn gõ. Nó chỉ kết nối tới GitHub để kiểm tra và tải bản cập nhật có chữ ký; không gửi nội dung gõ hay mã nhận dạng nào, và khi tắt việc kiểm tra thì hoàn toàn không dùng mạng.

VKey, Vikey và VieLotus-IM đã làm một phần việc này (tự nhận tiếng Anh, nhường khi layout là CJK), và mình học từ họ. Điểm khác là TrioKey đọc trạng thái bật/tắt của IME thay vì layout. So sánh: docs/COMPARISON.vi.md.

Đây là bản thử: chưa có bản dựng sẵn (phải tự build), mới thử trên Windows 11, chỉ có Telex. Mới có một bản dựng trước đây được gõ thử bằng tay, với tiếng Nhật; khay, cửa sổ cài đặt và chế độ admin hiện tại chưa từng chạy trên máy thật. Tiếng Trung và tiếng Hàn mới chỉ có unit test cho luật quyết định. Phần nhận biết Anh/Việt đạt 92,0% trên bộ test công khai của VieLotus-IM khi gõ không ngữ cảnh; một lỗi đã biết là phím s/f/r/x/j gõ lặp trong từ nó không biết (`passwd` → `paswd`).

Nếu bạn cũng gõ ba thứ tiếng mỗi ngày, mình rất muốn biết nó gõ sai những từ nào.

https://github.com/ved0el/triokey
