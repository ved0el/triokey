# TrioKey — gõ thử trên máy thật (M2–M5)

Agent chỉ biên dịch và chạy `cargo test`, chưa từng chạy `triokey.exe`. Phần gõ thử là việc của người dùng. Làm lần lượt từ trên xuống, ghi kết quả vào ô `Kết quả`.

**Phạm vi:** TrioKey nhường cho IME CJK (Chinese, Japanese, Korean). Tiếng Nhật (Microsoft IME, layout `0411`) đã được gõ thử tay trên một bản dựng trước đây: AT1 và AT2 dưới đây đạt (chưa điền vào các ô kết quả); bản hiện tại chưa gõ thử lại. Tiếng Trung và tiếng Hàn mới có luật được kiểm bằng unit test (`crates/triokey/src/ime.rs`), chưa ai gõ thử tay; các bước ở mục AT2-CK, làm khi có máy cài hai layout đó.

Tên tạm trước đây là EVJ. Nếu từng chạy bản thử tên cũ: **thoát `evj.exe` trước** (bản mới dùng mutex tên khác nên không nhận ra bản cũ đang chạy; hai bản sẽ sửa chữ của nhau). Lần chạy đầu, TrioKey chép `config.txt`, `force_vi.txt`, `force_en.txt` từ `%APPDATA%\evj` sang `%APPDATA%\triokey` nếu thư mục mới chưa có (không chép khi đang chạy quyền admin; `run_as_admin` về `false`, phải bật lại chế độ admin và `Khởi động cùng Windows` trong `Cài đặt…`). Thư mục cũ được để nguyên; cách dọn ở mục "Gỡ hẳn".

## Trước khi bắt đầu

1. **Thoát VKey** và mọi bộ gõ tiếng Việt khác (kể cả biểu tượng ở khay). Hai bộ gõ chạy cùng lúc sẽ sửa chữ của nhau.
2. Build (nếu chưa có): ở gốc repo chạy `mise x -- cargo build -p triokey --release --target-dir target/m4`.
3. Chạy: nhấp đúp `target\m4\release\triokey.exe` (cùng một file cho mọi mục dưới đây: mục tự khởi động và bản admin lấy đường dẫn của file đang chạy). Không có cửa sổ nào hiện ra; ở khay hệ thống xuất hiện chữ `V` đơn sắc, nền trong suốt (có thể nằm trong nhóm biểu tượng ẩn `^`).
   - Lần chạy đầu TrioKey tạo `%APPDATA%\triokey\config.txt` với giá trị mặc định. Có thể tự tạo thêm `force_vi.txt` / `force_en.txt` cùng thư mục (mỗi dòng một từ, viết theo phím gõ, `#` là chú thích); TrioKey đọc chúng lúc khởi động.
4. Biểu tượng khay: `V` = gõ tiếng Việt, `E` = phím đi thẳng (đã tắt, hoặc đang nhường IME CJK; tooltip ghi tên IME: `IME Nhật`, `IME Trung` hoặc `IME Hàn`). Di chuột lên biểu tượng để xem lý do.
   - Nhấp trái: bật/tắt. Nhấp phải: menu `Tiếng Việt` (có dấu tick khi bật) · `Cài đặt…` · `Thoát`.
   - Phím tắt bật/tắt mặc định: bấm Ctrl+Shift rồi nhả, không kèm phím nào khác.

**Thoát:** nhấp phải biểu tượng khay → `Thoát`. Nếu bàn phím loạn và không với tới khay: Ctrl+Shift+Esc → Task Manager → kết thúc `triokey.exe`. Windows tự gỡ hook khi tiến trình dừng.

**Gỡ:** xem mục "Gỡ hẳn" ở cuối file (M4 có thể đã tạo mục tự khởi động, tác vụ và bản admin).

---

## AT1 — Layout Nhật, IME đóng, gõ tiếng Việt (ĐIỀU KIỆN DỪNG)

**Trượt AT1 ở Notepad hoặc Brave thì dừng toàn bộ và báo lại** (spec 6.1): khi đó cách "xoá lùi + chèn Unicode" không dùng được ở layout `0411`.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 1a | Notepad, layout `0409` (ENG), gõ `tieesng vieejt ` | `tiếng việt ` | |
| 1b | Notepad, chuyển sang layout `0411`, IME ở chế độ direct input (`A`), gõ `tieesng vieejt ` | `tiếng việt ` | |
| 1c | Brave (ô soạn thảo bất kỳ, không phải thanh địa chỉ), layout `0411`, IME đóng, gõ `tieesng vieejt ` | `tiếng việt ` | |
| 1d | Notepad: `Vieejt VIEEJT DDuowngf Windows HTML ` | `Việt VIỆT Đường Windows HTML ` | |
| 1e | Notepad, bật Caps Lock, gõ `vieejt ` rồi tắt Caps Lock | `VIỆT ` (nếu ra `việt`: Caps Lock bị đọc sai, ghi lại) | |
| 1f | Notepad: gõ `terrible` rồi Enter | `terrible` và xuống dòng thật (không phải ký tự lạ) | |
| 1g | Notepad: gõ `tiee`, nhấp chuột vào giữa một dòng khác, gõ `s` | chỉ thêm `s` tại chỗ nhấp, không xoá chữ nào | |
| 1h | Notepad: gõ `tieesng`, ⌫ ⌫, gõ `ng ` | `tiếng ` | |

## AT2 — Bật/tắt hiragana, không chạm phím tắt của TrioKey

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 2a | Notepad, layout `0411`: bật hiragana, gõ `nihongo` + Enter; tắt hiragana, gõ `tieesng ` | `にほんご` rồi `tiếng `; khay đổi `E` ↔ `V` | |
| 2b | Như 2a trong Brave | như trên | |
| 2c | Như 2a trong một terminal (Windows Terminal) | ghi lại, không chặn | |
| 2d | Như 2a trong một ứng dụng Office (Word/Excel) | ghi lại, không chặn | |
| 2e | Như 2a trong một ứng dụng Electron (VS Code, Slack…) | ghi lại, không chặn | |

Ứng dụng không cho đọc trạng thái IME thì TrioKey nhường (khay `E`) khi ở layout Nhật; ở layout ENG vẫn gõ Việt được. Đó là hành vi chấp nhận được, chỉ cần ghi lại.

## AT3 — Phím đầu tiên ngay sau khi bật/tắt hiragana

Notepad, layout `0411`. Mỗi lần: bấm phím bật/tắt hiragana rồi gõ **ngay** một chữ (không chờ). Làm 10 lần chiều tắt rồi gõ `a` (mong đợi chữ `a` Latin, không phải `あ`), 10 lần chiều bật rồi gõ `a` (mong đợi `あ` đang soạn). Xoá dòng giữa các lần.

| Chiều | Số lần đúng / 20 | Ghi chú |
| --- | --- | --- |
| tắt hiragana → phím đầu là Latin/Telex | / 10 | |
| bật hiragana → phím đầu vào IME | / 10 | |

Đạt khi 20/20. Trượt dù đã có thời gian chờ 30 ms thì dừng và báo lại (spec 6.1).

## AT2-CK — Lặp lại AT2/AT3 với tiếng Trung và tiếng Hàn (CHƯA CHẠY)

Chưa ai làm mục này: máy thử chỉ có layout `0409` và `0411`. Làm khi có máy cài **Chinese (Simplified) – Microsoft Pinyin** (layout `0804`) và **Korean – Microsoft IME** (layout `0412`). Không chặn mốc nào; ghi lại kết quả.

Luật trong code (khác tiếng Nhật): với Trung/Hàn, TrioKey nhường khi IME mở **và** conversion mode có cờ NATIVE (đang gõ 中 / 한); ở chế độ chữ Latin (英 / A) thì gõ Telex. Khay khi nhường là `E`, tooltip `TrioKey: Tiếng Anh — đang nhường IME Trung` (layout `0804`) hoặc `… IME Hàn` (layout `0412`). Phím chuyển chế độ mặc định: Pinyin là Shift, Korean là phím Han/Eng hoặc Alt phải.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| CK1 | Notepad, layout `0804`, Pinyin ở chế độ 英, gõ `tieesng vieejt ` | `tiếng việt `; khay `V` | |
| CK2 | Notepad, layout `0804`: chuyển 中, gõ `nihao` + phím cách; chuyển 英, gõ `tieesng ` | `你好` rồi `tiếng `; khay đổi `E` ↔ `V`; không chạm phím tắt của TrioKey | |
| CK3 | Như CK2 trong Brave | như trên | |
| CK4 | Notepad, layout `0804`: bấm phím chuyển rồi gõ **ngay** `a`. 10 lần chiều 中 → 英, 10 lần chiều 英 → 中 | chiều về 英: chữ `a` Latin; chiều sang 中: `a` nằm trong khung soạn của Pinyin. Ghi số lần đúng / 20 | |
| CK5 | Notepad, layout `0412`, Korean IME ở chế độ `A`, gõ `tieesng vieejt ` | `tiếng việt `; khay `V` | |
| CK6 | Notepad, layout `0412`: chuyển 한, gõ `gksrmf`; chuyển `A`, gõ ` tieesng ` | `한글` rồi ` tiếng `; khay đổi `E` ↔ `V` | |
| CK7 | Như CK6 trong Brave | như trên | |
| CK8 | Notepad, layout `0412`: bấm phím chuyển rồi gõ **ngay** `r`. 10 lần chiều 한 → `A`, 10 lần chiều `A` → 한 | chiều về `A`: chữ `r` Latin; chiều sang 한: `ㄱ`. Ghi số lần đúng / 20 | |
| CK9 | Ở một trong hai layout, mở `Cài đặt…`, bỏ `Nhường khi IME Nhật/Trung/Hàn đang bật` | khay về `V` dù IME đang ở 中 / 한; chọn lại → `E` | |

Các dòng 2c–2e của AT2 (terminal, Office, Electron) lặp lại y như vậy với hai layout này: chỉ ghi lại, không chặn.

Nếu CK2/CK6 trượt (TrioKey sửa chữ trong lúc IME đang soạn, hoặc không gõ được Việt ở chế độ Latin): ghi layout, ứng dụng, và khay đang hiện gì; cờ NATIVE có thể không phản ánh đúng chế độ ở IME đó.

## AT5 — Ô có tự điền

Chưa có xử lý huỷ gợi ý (spec 5.5); lần này chỉ đo hiện trạng để chọn cách sửa.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 5a | Thanh địa chỉ Brave, gõ `tieesng vieejt` 20 lần (mỗi lần xoá trắng ô trước) | đúng từng ký tự 20/20 | / 20 |
| 5b | Một ô Excel, gõ `tieesng vieejt` vài lần | ghi lại, không chặn | |

## AT6 — Không đổi danh sách bàn phím của Windows

| Bước | Thao tác | Kết quả |
| --- | --- | --- |
| 6a | Trước khi chạy TrioKey: PowerShell `reg query "HKCU\Keyboard Layout\Preload"` và `Get-WinUserLanguageList`, lưu kết quả | |
| 6b | Chạy TrioKey, gõ một lúc, Thoát, chạy lại hai lệnh trên | giống hệt 6a? |

(Phần "bật tự khởi động" của AT6: xem M4, bước 4a–4e.)

## AT7 — Thời gian callback của hook

Gõ liên tục ít nhất 500 phím (văn bản bình thường, không bật/tắt IME), rồi mở `Cài đặt…`, đọc dòng xám dưới cùng `Hook p99: gửi A µs · không gửi B µs (M phím)`. Số đo chỉ nằm trong bộ nhớ, mỗi loại tính trên 1024 phím gần nhất, không tính phím đầu tiên sau khi bật/tắt IME. "gửi" = phím mà TrioKey thay chữ bằng `SendInput` (thời gian gồm cả lời gọi đó); "không gửi" = phím để đi qua.

Đạt khi p99 "không gửi" < 1000 µs. p99 "gửi" chỉ ghi lại để biết (`SendInput` vẫn nằm trong callback).

| p99 không gửi (µs) | p99 gửi (µs) | Số phím | Đạt? |
| --- | --- | --- | --- |
| | | | |

## AT8 — Ô mật khẩu Win32

Cần một ô mật khẩu Win32 cổ điển: control lớp `Edit` (hoặc `RichEdit…`) có style `ES_PASSWORD`, thường gặp ở hộp đăng nhập của ứng dụng desktop cũ. Nếu không chắc ô đó có phải loại này, ghi tên ứng dụng vào ô kết quả. Gõ `has`, bấm hiện mật khẩu nếu có.

| Thao tác | Mong đợi | Kết quả |
| --- | --- | --- |
| gõ `has` | nhận đúng `has` (không thành `há`) | |

Ô mật khẩu trong trình duyệt / Electron không nhận ra được (spec 5.6); không thuộc AT8.

---

# M4 — Cài đặt, tự khởi động, chế độ admin

Dùng cùng file `target\m4\release\triokey.exe` như ở đầu trang. Nếu vừa build lại: Thoát bản đang chạy rồi chạy lại file đó.

## Khay và menu

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 1a | Nhìn biểu tượng khay ở 100% và 150% (Settings → Display → Scale; đổi xong nên đăng xuất/vào lại) | chữ `V` / `E` đơn sắc, nền trong suốt, nét sắc, không mờ, không bị cắt. Ảnh mẫu: `docs/icons-preview.png` | |
| 1b | Settings → Personalization → Colors: đổi giữa Dark và Light (chế độ Windows) | biểu tượng đổi trắng ↔ gần đen ngay, không cần chạy lại | |
| 1c | Nhấp trái biểu tượng | `V` ↔ `E`; tooltip `TrioKey: Tiếng Việt` / `TrioKey: Tiếng Anh — đã tắt` | |
| 1d | Layout `0411`, bật hiragana | `E`, tooltip `TrioKey: Tiếng Anh — đang nhường IME Nhật` | |
| 1e | Nhấp phải | menu `Tiếng Việt` (tick khi bật) · `Cài đặt…` · gạch ngang · `Thoát`; không còn dòng `Hook p99` | |
| 1f | Task Manager → Processes → `Windows Explorer` → Restart | thanh taskbar hiện lại, biểu tượng TrioKey trở lại trong vài giây, nhấp trái/phải vẫn được | |
| 1g | `Thoát` | biểu tượng mất ngay, không để lại biểu tượng "ma" phải rê chuột mới mất | |

## Cửa sổ cài đặt

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 2a | Nhấp phải → `Cài đặt…` | cửa sổ `TrioKey – Cài đặt` giữa màn hình, chữ tiếng Việt đủ dấu, không mờ ở 150%, không chữ nào bị cắt; có ô `Tự động cập nhật`, nút `Kiểm tra bản mới` và một dòng kết quả bên dưới (hai dòng chữ, không bị cắt); dưới cùng có `v0.1.0` và dòng xám `Hook p99: …` (cập nhật mỗi giây). Ô chọn, nút và hộp chọn có kiểu hiện hành của Windows 11 (không phải kiểu xám cổ điển), chữ Segoe UI, khoảng cách đều | |
| 2b | Mở `Cài đặt…` lần nữa khi cửa sổ đang mở | không mở cửa sổ thứ hai, cửa sổ cũ nổi lên | |
| 2c | Vừa mở cửa sổ, bấm phím cách ngay; rồi Tab / Shift+Tab / phím cách; Tab tới một ô khác, Alt+Tab đi rồi về | phím cách đầu tiên bật/tắt `Bật gõ tiếng Việt` (ô này có focus sẵn; chọn lại cho bật); Tab đi qua lần lượt các ô; sau Alt+Tab focus vẫn ở ô cũ | |
| 2d | Bỏ chọn `Bật gõ tiếng Việt` | khay thành `E` ngay; `config.txt` có `enabled = false`; chọn lại → `V` | |
| 2d2 | Để cửa sổ mở, nhấp trái biểu tượng khay (hoặc bấm phím tắt) | ô `Bật gõ tiếng Việt` đổi theo ngay; `config.txt` không đổi | |
| 2e | Phím tắt `Alt+Z`: bấm Alt+Z trong Notepad, nhả cả hai, rồi gõ `a` | bật/tắt (khay đổi), Notepad không nhận `z`; chữ `a` vào văn bản (thanh menu của Notepad **không** được kích hoạt); Ctrl+Shift không còn bật/tắt | |
| 2e2 | Giữ Alt+Z 2 giây rồi nhả; lần nữa nhưng nhả Alt trước, Z sau | khay đổi đúng **một** lần mỗi lần giữ, không nhấp nháy; không có chữ `z` nào lọt vào Notepad | |
| 2e3 | Phím tắt `Ctrl+Space`: bấm Ctrl+Space trong Notepad; giữ 2 giây; rồi bấm Ctrl+Shift+Space | bật/tắt một lần mỗi lần bấm, không có dấu cách nào vào Notepad; Ctrl+Shift+Space không bật/tắt | |
| 2e4 | Chọn `Không dùng` | không phím nào bật/tắt, nhấp khay vẫn được. Trả về `Ctrl+Shift` | |
| 2f | Chọn `hoà, thuý`, gõ ngay (không chạy lại) `hoaf thuys ` | `hoà thuý `; chọn lại `hòa, thúy` → `hòa thúy ` | |
| 2g | Bỏ `Tự giữ nguyên từ tiếng Anh`, gõ vài từ tiếng Anh như `songs ` | hành xử khác lúc bật (ghi lại kết quả); chọn lại | |
| 2h | Bỏ `Nhường khi IME Nhật/Trung/Hàn đang bật`, bật hiragana | khay về `V` trong ~0,1 s (TrioKey không nhường nữa); chọn lại → `E` | |
| 2i | `Mở thư mục cấu hình…` (bản thường) | Explorer mở `%APPDATA%\triokey`; `config.txt` phản ánh mọi thay đổi ở trên | |
| 2j | Thoát, chạy lại TrioKey, mở `Cài đặt…` | các lựa chọn ở trên còn nguyên | |
| 2j2 | Bản build với khoá giữ chỗ (repo nay đã có khoá thật: build riêng sau khi tạm thay nội dung `crates/triokey/update-pubkey.txt` bằng một dòng `#`, **không commit**, rồi trả lại bằng `git checkout -- crates/triokey/update-pubkey.txt`): nhìn ô `Tự động cập nhật` và nút `Kiểm tra bản mới` | cả hai bị mờ, dòng bên dưới ghi `Bản build này không có khoá cập nhật nên không tự cập nhật.`; `config.txt` có `auto_update = true` và `last_update_check = 0` (không đổi dù để chạy lâu) | |
| 2k | (Nếu có hai màn hình khác tỉ lệ, hoặc đổi Scale khi cửa sổ đang mở) kéo cửa sổ sang màn hình kia | cửa sổ đổi cỡ theo màn hình mới, chữ sắc nét, không ô nào bị cắt hay lệch; các lựa chọn giữ nguyên | |
| 2l | Thoát. Mở `config.txt` bằng Notepad, Save As với Encoding `UTF-16 LE`. Chạy lại, mở `Cài đặt…`, đổi một ô | mỗi lần mở cửa sổ có cảnh báo "Không đọc được config.txt…"; TrioKey chạy với giá trị mặc định; file `config.txt` **không** bị ghi đè. Lưu lại file dạng `UTF-8` (có BOM cũng được), chạy lại: hết cảnh báo, dòng đầu `enabled = …` vẫn được đọc | |

## Tự khởi động (chế độ thường) và AT6

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 4a | Trước khi bật: `reg query "HKCU\Keyboard Layout\Preload"` và `Get-WinUserLanguageList`, lưu lại | | |
| 4b | Chọn `Khởi động cùng Windows` | `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v TrioKey` ra `"<đường dẫn triokey.exe đang chạy>"` (có ngoặc kép) | |
| 4c | Đăng xuất, đăng nhập lại | TrioKey tự chạy (khay `V`) | |
| 4d | Bỏ chọn `Khởi động cùng Windows` | lệnh `reg query` ở 4b báo không tìm thấy | |
| 4e | Chạy lại hai lệnh ở 4a | giống hệt 4a (AT6) | |

Ô này hiện đúng trạng thái registry, không theo `config.txt`: lần đầu chạy TrioKey không tự ghi gì vào Run, nên ô trống cho tới khi bạn chọn. Tắt TrioKey ở tab Startup của Task Manager không xoá giá trị đó nên ô vẫn được chọn.

## AT9 — Chế độ admin (cần UAC)

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 9a | Chọn `Chạy quyền admin…` | một hộp UAC cho `triokey.exe` | |
| 9b | Bấm **No** ở UAC | ô trở lại trống, không có thông báo lỗi, không có gì thay đổi | |
| 9c | Chọn lại, bấm **Yes** | hộp hỏi "Đã bật chế độ admin… Chạy bản admin ngay bây giờ?" hiện ra. **Để nguyên hộp này** tới bước 9g | |
| 9d | `schtasks /Query /TN TrioKey /XML` | có `<RunLevel>HighestAvailable</RunLevel>`, `<Command>C:\Program Files\TrioKey\triokey.exe</Command>`, `<Priority>4</Priority>`, `<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>` | |
| 9e | `icacls "C:\Program Files\TrioKey"` | chỉ Administrators/SYSTEM/TrustedInstaller có quyền ghi (`F`/`M`); Users chỉ `RX`. Thư mục không còn `task.xml` | |
| 9f | `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v TrioKey` | không tìm thấy (tác vụ thay thế) | |
| 9g | Trong thông báo, bấm **Yes** ("Chạy bản admin ngay bây giờ?") | bản đang chạy thoát, sau ≤ 3 s biểu tượng `V` lại hiện; Task Manager → Details: `triokey.exe` ở `C:\Program Files\TrioKey`, cột Elevated = Yes, chỉ có **một** `triokey.exe` | |
| 9h | Mở Notepad bằng "Run as administrator", gõ `tieesng vieejt ` | `tiếng việt ` (AT9) | |
| 9i | Chạy thêm `target\m4\release\triokey.exe` (bản thường) | sau ~3 s nó tự thoát, vẫn chỉ một biểu tượng khay | |
| 9j | Đăng xuất, đăng nhập lại | bản admin tự chạy, không hỏi UAC; 9h vẫn đạt | |
| 9j2 | Từ bản admin: Task Manager → `Windows Explorer` → Restart | biểu tượng TrioKey trở lại trong vài giây (bản quyền admin vẫn nhận được thông báo taskbar tạo lại) | |
| 9j3 | Từ bản admin: `Cài đặt…` → `Mở thư mục cấu hình…` | **không** mở Explorer; một hộp thoại ghi đường dẫn `%APPDATA%\triokey` | |
| 9j4 | Từ bản admin: đổi `Kiểu bỏ dấu`, xem `config.txt` | `tone_new` đổi theo (bản admin vẫn ghi được file đã có sẵn). Trả lại như cũ | |
| 9k | Từ bản admin: `Cài đặt…` → bỏ chọn `Chạy quyền admin…` | không hỏi UAC và không có tiến trình `triokey.exe` thứ hai (phần admin chạy ngay trong bản này); thông báo đã tắt, thư mục xoá sau khi khởi động lại máy, nhắc Thoát và chạy lại bản thường | |
| 9k2 | Ngay sau 9k, vẫn trong bản admin đó: chọn lại `Chạy quyền admin…` | báo "Không bật được chế độ admin: không chép được triokey.exe vào Program Files (mã 3)", ô trở lại trống, không có tác vụ nào được tạo (file của bản admin đã được đổi tên chờ xoá; phải bật lại từ bản thường) | |
| 9l | `schtasks /Query /TN TrioKey` | báo không tìm thấy | |
| 9m | Thoát, chạy bản thường, chọn lại `Khởi động cùng Windows` nếu muốn; khởi động lại máy | `C:\Program Files\TrioKey` không còn | |
| 9m2 | Thoát. Đổi tên `%APPDATA%\triokey\config.txt` thành tên khác, chạy bản thường bằng "Run as administrator", mở `Cài đặt…`, đổi một ô | không có `config.txt` nào được tạo; cảnh báo "Không lưu được config.txt…" khi mở cửa sổ và sau khi đổi. Thoát, đổi tên file lại như cũ | |
| 9m3 | Chỉ khi có sẵn một máy không tách token: Windows Sandbox (có lẽ, chưa kiểm), tài khoản `Administrator` dựng sẵn, hoặc UAC tắt hẳn (`EnableLUA = 0`; kéo thanh trượt UAC xuống thấp nhất **không** đủ, token vẫn bị tách). Đừng tắt UAC trên máy chính chỉ để thử. Chạy TrioKey lần đầu ở đó, mở `Cài đặt…`, đổi một ô, bấm `Mở thư mục cấu hình…` | `%APPDATA%\triokey\config.txt` được tạo và đổi theo; **không** có cảnh báo "Không lưu được config.txt…"; Explorer mở thư mục (ở đó không có phía quyền thường nào nên TrioKey xử sự như bản thường) | |
| 9n | (Từ bản thường) bật chế độ admin, bấm **No** ở câu "chạy ngay"; xem ô `Khởi động cùng Windows`; rồi bỏ chọn `Chạy quyền admin…` ngay trong bản thường | sau khi bấm No: `Khởi động cùng Windows` được chọn và mờ đi. Khi bỏ chọn: một UAC; thông báo đã tắt và xoá `C:\Program Files\TrioKey`; thư mục mất ngay; nếu `Khởi động cùng Windows` đã bật trước đó thì giá trị Run `TrioKey` trở lại | |

Chưa ai kiểm (agent không chạy được): `schtasks /Run` từ bản thường có chạy được tác vụ quyền cao nhất không (9g), schtasks có nhận file XML UTF-16 không (9c), phím "che" sau Alt+Z có chặn được thanh menu không (2e), biểu tượng có trở lại sau khi Explorer khởi động lại không (1f, 9j2), và cửa sổ cài đặt trông ra sao với manifest mới (2a, 2k). Nếu 9c báo "Không bật được chế độ admin", ghi lại. Chế độ admin giả định bạn là admin của máy và đồng ý UAC bằng chính tài khoản mình.

# M5 — Tự cập nhật (AT10)

Làm được từ giờ: khoá ký thật đã có trong repo và bản v0.1.0 đã được phát hành trên GitHub (2026-11-25). Agent chưa từng gửi một yêu cầu mạng nào và chưa từng thay một file exe đang chạy: mọi dòng dưới đây đều chưa ai kiểm.

**Chuẩn bị một bản "cũ":** sửa tạm `version` trong `crates/triokey/Cargo.toml` xuống thấp hơn bản đã phát hành (ví dụ `0.0.9`, **không commit**), build `mise x -- cargo build -p triokey --release --target-dir target/m4`, rồi trả `Cargo.toml` và `Cargo.lock` lại như cũ (`git checkout -- crates/triokey/Cargo.toml Cargo.lock`). Chép `target\m4\release\triokey.exe` vào một thư mục thử riêng (ví dụ `D:\tmp\tk`) và chạy từ đó. Trước mỗi bước, xoá dòng `last_update_check` trong `config.txt` khi TrioKey đang tắt, để nó kiểm tra lại ngay.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 10a | Bản build bình thường (cùng phiên bản với bản phát hành mới nhất), mở `Cài đặt…`, bấm `Kiểm tra bản mới` | dòng kết quả: `Đang kiểm tra…` rồi `Đang dùng bản mới nhất (vX.Y.Z).`; cửa sổ không đơ trong lúc chờ; `config.txt` có `last_update_check` khác 0 | |
| 10b | Chạy bản "cũ" từ thư mục thử, chờ khoảng một phút (không bấm gì) | thông báo `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau`; trong thư mục có `triokey.exe` (mới: SHA-256 khớp `SHA256SUMS.txt` của bản phát hành) và `triokey.old.exe`, không còn `triokey.new.exe`; gõ tiếng Việt vẫn bình thường trong lúc tải và sau đó | |
| 10c | Ngay sau 10b: mở `Cài đặt…`, bấm `Kiểm tra bản mới`; nhấp phải khay | dòng kết quả `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau.`; menu khay **không** có dòng cập nhật | |
| 10d | `Thoát`, chạy lại `triokey.exe` trong thư mục thử, mở `Cài đặt…` | góc dưới ghi phiên bản mới; `triokey.old.exe` đã mất; nếu `Khởi động cùng Windows` đang bật thì vẫn bật | |
| 10e | Bản "cũ" lần nữa (chép lại), bỏ chọn `Tự động cập nhật`, `Thoát`, xoá dòng `last_update_check`, chạy lại và để 3 phút. Mở Resource Monitor → Network, lọc `triokey.exe` | không có kết nối mạng nào, không có thông báo, exe không đổi; `config.txt` có `auto_update = false`. Bấm `Kiểm tra bản mới` thì vẫn kiểm tra (và ở bản thường thì cài) | |
| 10f | Ngắt mạng (tắt Wi-Fi), bấm `Kiểm tra bản mới` | sau tối đa vài chục giây: `Chưa kiểm tra được: không tải được từ GitHub (mất mạng, hoặc chưa có bản phát hành nào).`; gõ tiếng Việt không bị ảnh hưởng; `last_update_check` không đổi | |
| 10g | Chép bản "cũ" vào một thư mục tài khoản thường không ghi được (tạo bằng quyền admin, ví dụ `C:\Program Files\tk-test`), chạy **không** nâng quyền | thông báo `Có bản mới X.Y.Z`; menu khay có `Có bản X.Y.Z — mở trang tải về`; bấm thì trình duyệt mở trang phát hành; thư mục không có file mới nào. Dọn: xoá thư mục thử bằng quyền admin | |
| 10h | Từ bản "cũ" trong thư mục thử: bật `Chạy quyền admin…`, chọn chạy bản admin ngay, chờ khoảng một phút | thông báo `Có bản X.Y.Z — bấm để cập nhật`; `C:\Program Files\TrioKey\triokey.exe` **chưa** đổi và thư mục đó không có file mới nào (bản admin không tự cài; nó tải file exe vào bộ nhớ để kiểm chữ ký trước khi báo); `last_update_check` trong `config.txt` **không** đổi (bản admin không tự ghi file này); menu khay có dòng `Có bản X.Y.Z — bấm để cập nhật`; dòng kết quả trong `Cài đặt…` ghi `Có bản X.Y.Z. Bấm vào biểu tượng khay → …` | |
| 10i | Bấm vào dòng đó trong menu khay (hoặc vào thông báo) | không có hộp UAC; thông báo `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau`; trong `C:\Program Files\TrioKey` có exe mới và `triokey.old.exe`; `triokey.exe` trong thư mục thử (bản thường) **không** bị đụng tới. Đăng xuất/vào lại: bản admin là phiên bản mới, `triokey.old.exe` đã mất | |
| 10j | Tắt chế độ admin. Nhấp phải bản "cũ" trong thư mục thử → Run as administrator | thông báo `Có bản mới X.Y.Z`; bấm vào thì hiện hộp thoại ghi địa chỉ trang phát hành (trình duyệt **không** tự mở); thư mục thử không có `triokey.new.exe` hay `triokey.old.exe`, exe không đổi | |
| 10k | Trong lúc 10b đang tải (hoặc ngay sau), xem dòng `Hook p99` trong `Cài đặt…` | p99 "không gửi" vẫn dưới 1000 µs | |

Chưa ai kiểm (agent không chạy được): WinHTTP có đi qua được hai lần chuyển hướng của GitHub không, proxy của máy có được dùng không, thông báo kiểu "balloon" hiện ra sao trên Windows 11 và bấm vào có tới TrioKey không, và việc đổi tên file exe đang chạy trong thư mục thường lẫn trong `Program Files`. Cũng chưa thử: thay tên `triokey.exe` trong một bước khi nó đang chạy (`ReplaceFileW`, hoặc hard link rồi đổi tên đè) có được không; nếu được thì bỏ được khoảng hở giữa hai lần đổi tên ghi ở quyết định 0007.

## Gỡ hẳn

1. Thoát TrioKey (nhấp phải khay → `Thoát`).
2. Nếu đã bật chế độ admin, PowerShell **quyền admin**: `schtasks /Delete /TN TrioKey /F` rồi `Remove-Item -Recurse "C:\Program Files\TrioKey"`.
3. `reg delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v TrioKey /f` (báo không tìm thấy cũng được).
4. Xoá `%APPDATA%\triokey` nếu muốn bỏ cấu hình.
5. Xoá thư mục chứa `triokey.exe`. Sau một lần tự cập nhật, trong đó có thể còn `triokey.old.exe` (bản trước); xoá cùng luôn.

TrioKey không ghi gì khác: không layout bàn phím, không danh sách ngôn ngữ, không mục nào trong `HKCU\Keyboard Layout`.

**Đồ thừa của bản thử tên cũ (EVJ).** TrioKey không tự xoá hay đổi các mục này; xoá tay nếu từng chạy bản tên cũ (lệnh nào báo không tìm thấy cũng được):

1. Thoát `evj.exe` nếu còn chạy.
2. Giá trị Run: `reg delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v EVJ /f`.
3. Nếu bản cũ từng bật chế độ admin, PowerShell **quyền admin**: `schtasks /Delete /TN EVJ /F` rồi `Remove-Item -Recurse "C:\Program Files\EVJ"`.
4. Thư mục cấu hình cũ `%APPDATA%\evj` (TrioKey đã chép ba file sang `%APPDATA%\triokey` ở lần chạy đầu).
