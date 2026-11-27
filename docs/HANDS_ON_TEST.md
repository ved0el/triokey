# TrioKey — gõ thử trên máy thật

Những gì kiểm được bằng `cargo test` thì CI đã chạy; các mục dưới đây chỉ kiểm được trên máy thật. Làm lần lượt từ trên xuống, ghi kết quả vào ô `Kết quả`.

**Phạm vi:** TrioKey nhường cho IME CJK (Chinese, Japanese, Korean). Tiếng Nhật dùng Microsoft IME, layout `0411`. Với tiếng Trung và tiếng Hàn, luật được kiểm bằng unit test (`crates/triokey/src/ime.rs`); các bước gõ tay ở mục AT2-CK, làm khi có máy cài hai layout đó.

## Trước khi bắt đầu

1. **Thoát mọi bộ gõ tiếng Việt khác** (kể cả biểu tượng ở khay). Hai bộ gõ chạy cùng lúc sẽ sửa chữ của nhau.
2. Build (nếu chưa có): ở gốc repo chạy `mise x -- cargo build -p triokey --release --target-dir target/m4`.
3. Chạy: nhấp đúp `target\m4\release\triokey.exe` (cùng một file cho mọi mục dưới đây: mục tự khởi động và bản admin lấy đường dẫn của file đang chạy). Không có cửa sổ nào hiện ra; ở khay hệ thống xuất hiện chữ `V` màu đỏ, nền trong suốt (đỏ nhạt trên taskbar tối, đỏ đậm trên taskbar sáng; có thể nằm trong nhóm biểu tượng ẩn `^`).
   - Lần chạy đầu TrioKey tạo `%APPDATA%\triokey\config.txt` với giá trị mặc định và `learned.txt` (chỉ có hai dòng chú thích `#`; nơi lưu các từ đã đổi bằng Backspace, xem AT12). Có thể tự tạo thêm `force_vi.txt` / `force_en.txt` cùng thư mục (mỗi dòng một từ, viết theo phím gõ, `#` là chú thích); TrioKey đọc chúng lúc khởi động.
4. Biểu tượng khay: `V` = gõ tiếng Việt, `E` = phím đi thẳng (đã tắt, hoặc đang nhường IME CJK; tooltip ghi tên IME: `IME Nhật`, `IME Trung` hoặc `IME Hàn`). Di chuột lên biểu tượng để xem lý do.
   - Nhấp trái: bật/tắt. Nhấp phải: menu `Tiếng Việt` (có dấu tick khi bật) · `Cài đặt…` · `Khởi động lại` · `Thoát`.
   - Phím tắt bật/tắt mặc định: bấm Ctrl+Shift rồi nhả, không kèm phím nào khác.

**Thoát:** nhấp phải biểu tượng khay → `Thoát`. Nếu bàn phím loạn và không với tới khay: Ctrl+Shift+Esc → Task Manager → kết thúc `triokey.exe`. Windows tự gỡ hook khi tiến trình dừng.

**Gỡ:** xem mục "Gỡ hẳn" ở cuối file (phần "Cài đặt, tự khởi động, chế độ admin" có thể đã tạo mục tự khởi động, tác vụ và bản admin).

---

## AT1 — Layout Nhật, IME đóng, gõ tiếng Việt (ĐIỀU KIỆN DỪNG)

**Trượt AT1 ở Notepad hoặc Brave thì dừng toàn bộ và báo lại**: khi đó cách "xoá lùi + chèn Unicode" không dùng được ở layout `0411`.

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

Đạt khi 20/20. Trượt dù đã có thời gian chờ 30 ms thì dừng và báo lại (`docs/ARCHITECTURE.md`, 3.3).

## AT2-CK — Lặp lại AT2/AT3 với tiếng Trung và tiếng Hàn

Làm khi có máy cài **Chinese (Simplified) – Microsoft Pinyin** (layout `0804`) và **Korean – Microsoft IME** (layout `0412`). Không chặn mốc nào; ghi lại kết quả.

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

TrioKey không có xử lý riêng cho ô tự điền (`docs/ARCHITECTURE.md`, mục 13); lần này chỉ đo hiện trạng.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 5a | Thanh địa chỉ Brave, gõ `tieesng vieejt` 20 lần (mỗi lần xoá trắng ô trước) | đúng từng ký tự 20/20 | / 20 |
| 5b | Một ô Excel, gõ `tieesng vieejt` vài lần | ghi lại, không chặn | |

## AT6 — Không đổi danh sách bàn phím của Windows

| Bước | Thao tác | Kết quả |
| --- | --- | --- |
| 6a | Trước khi chạy TrioKey: PowerShell `reg query "HKCU\Keyboard Layout\Preload"` và `Get-WinUserLanguageList`, lưu kết quả | |
| 6b | Chạy TrioKey, gõ một lúc, Thoát, chạy lại hai lệnh trên | giống hệt 6a? |

(Phần "bật tự khởi động" của AT6: xem mục "Tự khởi động", bước 4a–4e.)

## AT7 — Thời gian callback của hook

Gõ liên tục ít nhất 500 phím (văn bản bình thường, không bật/tắt IME), rồi mở `Cài đặt…`, chọn `Chế độ phát triển` (mặc định tắt), bấm `Chép thông tin debug`, dán vào Notepad và đọc dòng `hook: p99 A µs over N keys that sent text; B µs over M keys passed through`. Số đo chỉ nằm trong bộ nhớ, mỗi loại tính trên 1024 phím gần nhất, không tính phím đầu tiên sau khi bật/tắt IME. "gửi" = phím mà TrioKey thay chữ bằng `SendInput` (thời gian gồm cả lời gọi đó; `A` và `N` trong `sent text`); "không gửi" = phím để đi qua (`B` và `M` trong `passed through`).

Đạt khi p99 "không gửi" < 1000 µs. p99 "gửi" chỉ ghi lại để biết (`SendInput` vẫn nằm trong callback).

| p99 không gửi (µs) | p99 gửi (µs) | Số phím | Đạt? |
| --- | --- | --- | --- |
| | | | |

## AT8 — Ô mật khẩu Win32

Cần một ô mật khẩu Win32 cổ điển: control lớp `Edit` (hoặc `RichEdit…`) có style `ES_PASSWORD`, thường gặp ở hộp đăng nhập của ứng dụng desktop cũ. Nếu không chắc ô đó có phải loại này, ghi tên ứng dụng vào ô kết quả. Gõ `tieesng`, bấm hiện mật khẩu nếu có.

| Thao tác | Mong đợi | Kết quả |
| --- | --- | --- |
| gõ `tieesng` | nhận đúng `tieesng` (không thành `tiếng`) | |

Ô mật khẩu trong trình duyệt / Electron không nhận ra được (`docs/ARCHITECTURE.md`, 4.3); không thuộc AT8.

## AT11 — Engine mới: gõ lẫn tiếng Anh và tiếng Việt

Engine mới quyết định từng từ theo tần suất dùng của hai cách đọc và theo "chế độ dòng": dòng mới hơi nghiêng về tiếng Việt, mỗi từ đã gõ kéo dòng về một phía, Enter bắt đầu dòng mới. Mọi kết quả mong đợi dưới đây đã được kiểm bằng cách cho engine gõ đúng các phím đó.

Notepad, layout `0409`. **Mỗi bước bắt đầu ở một dòng mới** (bấm Enter trước). Sau một lần Backspace đổi từ (⌫ ra cách đọc khác), **đừng** gõ phím cách hay Enter: nhấp chuột ra chỗ khác rồi xoá dòng, nếu không TrioKey sẽ nhớ lần đổi đó (xem AT12). ⌫ dưới đây là bấm Backspace một lần ngay sau phím cách.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 11a | `tooi caanf push code leen server ` | `tôi cần push code lên server ` | |
| 11b | `I want to test this feature ` | giữ nguyên | |
| 11c | `this ` rồi ⌫ (engine chưa chắc) | `this ` rồi `thí` (dấu cách cũng mất) | |
| 11d | `tooi ` rồi ⌫ (engine chắc chắn) | `tôi ` rồi `tôi` (chỉ xoá dấu cách) | |
| 11e | `mixx ` rồi ⌫; dòng mới `offline passwd ` | `mix ` rồi `mix` (chỉ xoá dấu cách); `offline passwd ` giữ nguyên | |
| 11f | `off ` rồi ⌫ | `off ` rồi `of` | |
| 11g | `chaau As `; dòng mới `OS naof cungx dduwowcj ` | `châu Á `; `OS nào cũng được ` | |
| 11h | `Mac OS `; dòng mới `caif OS mowis ` | `Mac OS `; `cài OS mới ` | |
| 11i | `ko ddc ddaau `; dòng mới `dd/mm/yyyy `; dòng mới `dd ` rồi ⌫ | `ko đc đâu `; `dd/mm/yyyy ` giữ nguyên; `dd ` rồi `dd` (chỉ xoá dấu cách) | |
| 11j | `There is `; dòng mới `There ` rồi ⌫ | `Thể is ` (đã biết: dòng mới nghiêng về tiếng Việt); `Thể ` rồi `There` | |
| 11k | `tooi ddax nois vowis anh aays raats nhieeuf laanf sets ` rồi ⌫ | `… lần sét ` rồi `… lần sets` | |
| 11l | `TRUYEENF ` | `TRUYỀN ` (không phải `TRUỲÊN`) | |
| 11m | `w ` rồi ⌫ | `w ` rồi `ư` | |
| 11n | Lặp lại 11a và 11c trong Brave và trong một ứng dụng Electron (VS Code, Slack…) | như trên; ghi lại nếu khác | |

---

# Cài đặt, tự khởi động, chế độ admin

Dùng cùng file `target\m4\release\triokey.exe` như ở đầu trang. Nếu vừa build lại: Thoát bản đang chạy rồi chạy lại file đó.

## Khay và menu

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 1a | Nhìn biểu tượng khay ở 100% và 150% (Settings → Display → Scale; đổi xong nên đăng xuất/vào lại) | `V` đỏ / `E` xanh, nét bo tròn ở đầu, cạnh thẳng sắc, nền trong suốt, không mờ, không bị cắt. Ảnh mẫu: `docs/icons-preview.png` | |
| 1b | Settings → Personalization → Colors: đổi chế độ Windows (taskbar) giữa Dark và Light | đổi màu ngay, không cần chạy lại: `V` đỏ nhạt (Dark) ↔ đỏ đậm (Light); nhấp trái để tắt (khay `E`) rồi làm lại: `E` xanh nhạt ↔ xanh đậm | |
| 1c | Nhấp trái biểu tượng | `V` ↔ `E`; tooltip `TrioKey: Tiếng Việt` / `TrioKey: Tiếng Anh — đã tắt` | |
| 1d | Layout `0411`, bật hiragana | `E`, tooltip `TrioKey: Tiếng Anh — đang nhường IME Nhật` | |
| 1e | Nhấp phải | menu `Tiếng Việt` (tick khi bật) · `Cài đặt…` · gạch ngang · `Khởi động lại` · `Thoát`; không còn dòng `Hook p99` | |
| 1f | Task Manager → Processes → `Windows Explorer` → Restart | thanh taskbar hiện lại, biểu tượng TrioKey trở lại trong vài giây, nhấp trái/phải vẫn được | |
| 1g | `Thoát` | biểu tượng mất ngay, không để lại biểu tượng "ma" phải rê chuột mới mất | |
| 1h | Chạy lại, gõ thử, rồi nhấp phải → `Khởi động lại`; ngay sau đó gõ `tieesng ` trong Notepad | biểu tượng biến mất rồi hiện lại trong ≤ 3 s; Task Manager → Details chỉ có **một** `triokey.exe` (PID mới); `tiếng ` gõ đúng một lần, không bị sửa hai lần; các lựa chọn trong `Cài đặt…` giữ nguyên | |
| 1h2 | Đổi tên hoặc chuyển chỗ `triokey.exe` khi đang chạy (Explorer cho phép đổi tên file đang chạy), rồi `Khởi động lại` | hộp thoại "Không khởi động lại được…"; TrioKey **vẫn chạy** (biểu tượng còn, gõ được). Trả tên file lại như cũ | |

## Cửa sổ cài đặt

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 2a | Nhấp phải → `Cài đặt…` | cửa sổ `TrioKey – Cài đặt` giữa màn hình, chữ tiếng Việt đủ dấu, không mờ ở 150%, không chữ nào bị cắt; có ô `Tự động cập nhật`, nút `Kiểm tra bản mới` và (khi có kết quả) một dòng kết quả bên dưới, không bị cắt; dòng đầu của mục `Hệ thống` là `Ngôn ngữ` với hộp chọn `Tiếng Việt`, ô cuối là `Chế độ phát triển` (chưa chọn); **không** có nút `Mở thư mục cấu hình…` và dòng `Hook p99`. Một đường kẻ mảnh, rồi dòng cuối: bên trái chữ xám `TrioKey v` + phiên bản trong `crates/triokey/Cargo.toml` (không có tên tác giả, không có ©), bên phải liên kết `GitHub` (chữ xanh gạch dưới), cùng một đường chân chữ. Ba tiêu đề đậm `Gõ`, `Hệ thống`, `Cập nhật`, không khung, không viền nhóm. Ô chọn, nút và hộp chọn có kiểu hiện hành của Windows 11 (không phải kiểu xám cổ điển), chữ Segoe UI, khoảng cách đều | |
| 2b | Mở `Cài đặt…` lần nữa khi cửa sổ đang mở | không mở cửa sổ thứ hai, cửa sổ cũ nổi lên | |
| 2c | Vừa mở cửa sổ, bấm phím cách ngay; rồi Tab / Shift+Tab / phím cách; Tab tới một ô khác, Alt+Tab đi rồi về | phím cách đầu tiên bật/tắt `Bật gõ tiếng Việt` (ô này có focus sẵn; chọn lại cho bật); Tab đi qua lần lượt các ô; sau Alt+Tab focus vẫn ở ô cũ | |
| 2d | Bỏ chọn `Bật gõ tiếng Việt` | khay thành `E` ngay; `config.txt` có `enabled = false`; chọn lại → `V` | |
| 2d2 | Để cửa sổ mở, nhấp trái biểu tượng khay (hoặc bấm phím tắt) | ô `Bật gõ tiếng Việt` đổi theo ngay; `config.txt` không đổi | |
| 2e | Phím tắt `Alt+Z`: bấm Alt+Z trong Notepad, nhả cả hai, rồi gõ `a` | bật/tắt (khay đổi), Notepad không nhận `z`; chữ `a` vào văn bản (thanh menu của Notepad **không** được kích hoạt); Ctrl+Shift không còn bật/tắt | |
| 2e2 | Giữ Alt+Z 2 giây rồi nhả; lần nữa nhưng nhả Alt trước, Z sau | khay đổi đúng **một** lần mỗi lần giữ, không nhấp nháy; không có chữ `z` nào lọt vào Notepad | |
| 2e3 | Mở hộp `Phím tắt`; rồi Thoát, sửa `config.txt` thành `hotkey = ctrl_space` (giá trị của 0.3.0), chạy lại, mở `Cài đặt…`; bấm Ctrl+Space trong Notepad | danh sách có đúng bốn dòng `Ctrl+Shift` · `Alt+Shift` · `Alt+Z` · `Không dùng` (như bản macOS); sau khi sửa file: hộp hiện `Ctrl+Shift`, Ctrl+Shift bật/tắt, Ctrl+Space **không** bật/tắt và vào Notepad như thường | |
| 2e5 | Chọn `Alt+Shift`: trong Notepad bấm giữ Alt, bấm Shift, thả cả hai; rồi Alt+Shift+T; rồi Ctrl+Shift | lần đầu bật/tắt; Alt+Shift+T và Ctrl+Shift không bật/tắt; thanh menu của Notepad không bị kích hoạt. Nếu Windows đang dùng Left Alt+Shift để đổi ngôn ngữ nhập (Settings → Time & language → Typing → Advanced keyboard settings → Input language hot keys) thì nó cũng đổi ngôn ngữ: đó là của Windows | |
| 2e4 | Chọn `Không dùng` | không phím nào bật/tắt, nhấp khay vẫn được. Trả về `Ctrl+Shift` | |
| 2f | Chọn `hoà, thuý`, gõ ngay (không chạy lại) `hoaf thuys ` | `hoà thuý `; chọn lại `hòa, thúy` → `hòa thúy ` | |
| 2g | Bỏ `Tự giữ nguyên từ tiếng Anh`, gõ `windows ` | `uindows ` (Telex thuần); chọn lại → `windows ` | |
| 2h | Bỏ `Nhường khi IME Nhật/Trung/Hàn đang bật`, bật hiragana | khay về `V` trong ~0,1 s (TrioKey không nhường nữa); chọn lại → `E` | |
| 2i | Chọn `Chế độ phát triển`, bấm `Mở thư mục cấu hình…` (bản thường) | Explorer mở `%APPDATA%\triokey`; `config.txt` phản ánh mọi thay đổi ở trên | |
| 2i2 | Gõ vài từ (kể cả một từ đã nhớ ở AT12 nếu có), bấm `Chép thông tin debug`, dán vào Notepad | hộp thoại `Đã chép thông tin debug vào clipboard.`; 10 dòng tiếng Anh, mỗi dòng một ý (bắt đầu `TrioKey X.Y.Z (Windows 10.0.…, x64)`), có dòng `hook: p99 …` và `remembered words: <số>`; đường dẫn trong thư mục người dùng bắt đầu bằng `~`, không có tên người dùng ở đâu cả; không có chữ nào vừa gõ hay từ nào trong `learned.txt` | |
| 2j | Thoát, chạy lại TrioKey, mở `Cài đặt…` | các lựa chọn ở trên còn nguyên | |
| 2j2 | Bản build với khoá giữ chỗ (repo nay đã có khoá thật: build riêng sau khi tạm thay nội dung `crates/triokey/update-pubkey.txt` bằng một dòng `#`, **không commit**, rồi trả lại bằng `git checkout -- crates/triokey/update-pubkey.txt`): nhìn ô `Tự động cập nhật` và nút `Kiểm tra bản mới` | cả hai bị mờ, dòng bên dưới ghi `Bản build này không có khoá cập nhật nên không tự cập nhật.`; `config.txt` có `auto_update = true` và `last_update_check = 0` (không đổi dù để chạy lâu) | |
| 2k | (Nếu có hai màn hình khác tỉ lệ, hoặc đổi Scale khi cửa sổ đang mở) kéo cửa sổ sang màn hình kia | cửa sổ đổi cỡ theo màn hình mới, chữ sắc nét, không ô nào bị cắt hay lệch; các lựa chọn giữ nguyên | |
| 2l | Thoát. Mở `config.txt` bằng Notepad, Save As với Encoding `UTF-16 LE`. Chạy lại, mở `Cài đặt…`, đổi một ô | mỗi lần mở cửa sổ có cảnh báo "Không đọc được config.txt…"; TrioKey chạy với giá trị mặc định; file `config.txt` **không** bị ghi đè. Lưu lại file dạng `UTF-8` (có BOM cũng được), chạy lại: hết cảnh báo, dòng đầu `enabled = …` vẫn được đọc | |
| 2m | Nhìn biểu tượng ở góc trái thanh tiêu đề cửa sổ `Cài đặt…` và trên taskbar khi cửa sổ đang mở, ở 100% và 150%; nếu có hai màn hình khác tỉ lệ, kéo cửa sổ sang màn hình kia | ô vuông xanh bo góc với ba phím trắng, sắc nét, không mờ ở cả hai tỉ lệ; sau khi kéo sang màn hình kia vẫn sắc | |
| 2n | Chọn rồi bỏ chọn `Chế độ phát triển`, bằng chuột rồi bằng phím cách | khi chọn: cửa sổ cao thêm, góc trên bên trái đứng yên, ngay dưới ô đó (vẫn trong mục `Hệ thống`, mép trái thẳng hàng với chữ của ô) hiện hai nút cùng một hàng, cùng cỡ: `Mở thư mục cấu hình…` và `Chép thông tin debug`; bỏ chọn: hai nút biến mất, cửa sổ thấp lại. Cả hai lần con trỏ bàn phím vẫn ở ô `Chế độ phát triển` (phím cách bấm tiếp được ngay); `config.txt` có `dev_mode = true` rồi `dev_mode = false`. Tab khi chưa chọn không dừng ở nút đã ẩn | |
| 2o | Windows Settings → Personalization → Colors → "Choose your app mode" = **Light**, mở `Cài đặt…` | nền sáng; thanh tiêu đề **cùng một màu** với phần thân (không có dải màu khác ở trên), chữ tiêu đề đậm màu; viền cửa sổ là một đường xám nhạt vẫn thấy được trên nền desktop sáng | |
| 2p | Như 2o với app mode = **Dark** | nền tối; thanh tiêu đề cùng màu với thân, chữ tiêu đề sáng; mọi chữ (cả nhãn của ô chọn và nút chọn) đọc rõ, không có chữ đen trên nền tối; ô chọn đã tắt (ví dụ `Khởi động cùng Windows` ở chế độ admin) mờ hơn hẳn | |
| 2q | Để cửa sổ mở, đổi app mode Light ↔ Dark | cửa sổ đổi màu ngay (cả thanh tiêu đề), không phải đóng mở lại; vị trí, các lựa chọn và ô đang có con trỏ giữ nguyên | |
| 2r | Ở app mode Dark, mở hộp `Phím tắt` | danh sách thả xuống đọc rõ: chữ sáng trên nền tối (hoặc ít nhất đủ tương phản), dòng đang chọn được tô | |
| 2s | Bật tương phản cao (Settings → Accessibility → Contrast themes, hoặc Alt trái + Shift trái + Print Screen) khi cửa sổ đang mở; rồi tắt | khi bật: cửa sổ dùng giao diện hệ thống, màu do chủ đề tương phản quyết định, không còn màu tối/sáng riêng của TrioKey, thanh tiêu đề theo hệ thống; khi tắt: trở lại 2o/2p theo app mode | |
| 2t | Ở cả Light và Dark, dùng Tab đi qua các ô | ô đang có con trỏ có khung chấm quanh nhãn; nút và hộp chọn có viền focus của Windows; `Cập nhật tại đây` (khi có) là điểm dừng ngay sau `Kiểm tra bản mới`; `GitHub` là điểm dừng cuối; cả hai có khung chấm ôm sát chữ | |
| 2u | Bản thường: bấm liên kết `GitHub` ở dòng cuối; rồi Tab tới nó và nhấn phím cách, rồi Enter | trình duyệt mặc định mở `https://github.com/ved0el/triokey` mỗi lần; con trỏ chuột trên liên kết là bàn tay | |
| 2v | Bản admin (đã bật ở AT9, như 9j3): như 2u | **không** mở trình duyệt; mỗi lần một hộp thoại ghi địa chỉ `https://github.com/ved0el/triokey` | |
| 2w | Ở Light, Dark và tương phản cao, nhìn liên kết `GitHub` (và `Cập nhật tại đây` ở 10g/10h) | chữ gạch dưới, đọc rõ trên nền: xanh đậm ở Light, xanh nhạt ở Dark, màu liên kết của chủ đề ở tương phản cao; dấu nặng của `ạ`, `ậ` không bị đường gạch che | |
| 2x | Nhìn dòng kết quả cập nhật lúc mới mở (chưa kiểm tra), rồi bấm `Kiểm tra bản mới`, rồi (nếu có thể, ví dụ rút mạng) để có lỗi `Chưa kiểm tra được: không tải được từ GitHub.` | chưa có kết quả: một dòng trống được giữ sẵn dưới nút; `Đang kiểm tra…`, `Đang dùng bản mới nhất.` và lỗi không có mạng hiện trên đúng dòng đó, cửa sổ **không** đổi kích thước lần nào (cả khi bật `Chế độ phát triển`); chỉ một thông báo dài hai dòng (ví dụ lỗi khi cài dở) mới làm cửa sổ cao thêm; góc trên bên trái đứng yên, không chữ nào bị cắt | |
| 2y | Bấm `Kiểm tra bản mới` (đợi kết quả), rồi ở `Ngôn ngữ` chọn `English` bằng chuột | cửa sổ dựng lại ngay bằng tiếng Anh, góc trên bên trái đứng yên: tiêu đề `TrioKey – Settings`, `Enable Vietnamese`, `Shortcut` (`None` thay cho `Không dùng`), `Typing`, `Tone mark placement` (không đè lên `hòa, thúy`), `Keep English words as typed`, `Step aside while a Japanese/Chinese/Korean IME is on`, `System`, `Language` (hộp vẫn có `Tiếng Việt` và `English`), `Start with Windows`, `Run as administrator (types in admin windows too)`, `Developer mode`, `Updates`, `Update automatically`, `Check for updates`; dòng kết quả cập nhật cũng đổi sang tiếng Anh (ví dụ `You have the latest version.`); không chữ nào bị cắt, ở cả 100% và 150%; con trỏ bàn phím vẫn ở hộp `Language`; `config.txt` có `language = en` | |
| 2y2 | Rê chuột lên biểu tượng khay, nhấp phải, bật hiragana (layout `0411`) | tooltip `TrioKey: Vietnamese` / `TrioKey: English — off` / `TrioKey: English — yielding to the Japanese IME`; menu `Vietnamese` · `Settings…` · `Restart` · `Quit` (không có biểu tượng) | |
| 2y3 | Bật `Developer mode`, bấm `Copy debug info`; ở bản admin: `Open config folder…` và liên kết `GitHub` | hộp thoại bằng tiếng Anh (`The debug information is on the clipboard.`, `TrioKey runs as administrator, so it does not open …`); nội dung đã chép vẫn như 2i2 (tiếng Anh, dòng `update:` vẫn bằng tiếng Việt) | |
| 2y4 | `Restart` (hoặc Thoát rồi chạy lại) | menu, tooltip và cửa sổ vẫn bằng tiếng Anh | |
| 2y5 | Focus ở hộp `Language`, bấm mũi tên lên/xuống vài lần (hộp đang đóng); rồi Alt+↓ mở danh sách, bấm mũi tên lên/xuống (danh sách vẫn mở), Esc; mở lại, chọn `Tiếng Việt` bằng Enter; lần sau chọn bằng chuột | hộp đóng: mỗi lần đổi, cửa sổ dựng lại đúng ngôn ngữ đang chọn, không treo, focus vẫn ở hộp đó nên mũi tên bấm tiếp được; danh sách mở: mũi tên chỉ đổi dòng tô, danh sách **không** đóng, cửa sổ chưa đổi; Esc: không đổi gì; Enter hoặc nhấp chuột: danh sách đóng rồi cửa sổ dựng lại; cuối cùng mọi chữ trở lại tiếng Việt (như 2a) và `config.txt` có `language = vi` | |

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
| 9j3 | Từ bản admin: `Cài đặt…` → chọn `Chế độ phát triển` → `Mở thư mục cấu hình…` | **không** mở Explorer; một hộp thoại ghi đường dẫn `%APPDATA%\triokey` | |
| 9j3b | Từ bản admin: `Chép thông tin debug`, dán vào Notepad | dòng `copy: elevated; admin-mode copy yes`; không có tên người dùng | |
| 9j4 | Từ bản admin: đổi `Kiểu bỏ dấu`, xem `config.txt` | `tone_new` đổi theo (bản admin vẫn ghi được file đã có sẵn). Trả lại như cũ | |
| 9j5 | Từ bản admin: nhấp phải → `Khởi động lại` | không hỏi UAC; sau ≤ 3 s biểu tượng hiện lại; Task Manager → Details: **một** `triokey.exe` ở `C:\Program Files\TrioKey`, Elevated = Yes, PID mới; 9h vẫn đạt | |
| 9j6 | Thoát bản admin; chạy `target\m4\release\triokey.exe` bằng "Run as administrator"; nhấp phải biểu tượng | menu **không** có `Khởi động lại` (bản quyền cao nằm ở thư mục người dùng ghi được); `Thoát` vẫn có. Thoát, chạy lại bản admin hoặc bản thường | |

| 9k | Từ bản admin: `Cài đặt…` → bỏ chọn `Chạy quyền admin…` | không hỏi UAC và không có tiến trình `triokey.exe` thứ hai (phần admin chạy ngay trong bản này); thông báo đã tắt, thư mục xoá sau khi khởi động lại máy, nhắc Thoát và chạy lại bản thường | |
| 9k2 | Ngay sau 9k, vẫn trong bản admin đó: chọn lại `Chạy quyền admin…` | báo "Không bật được chế độ admin: không chép được triokey.exe vào Program Files (mã 3)", ô trở lại trống, không có tác vụ nào được tạo (file của bản admin đã được đổi tên chờ xoá; phải bật lại từ bản thường) | |
| 9l | `schtasks /Query /TN TrioKey` | báo không tìm thấy | |
| 9m | Thoát, chạy bản thường, chọn lại `Khởi động cùng Windows` nếu muốn; khởi động lại máy | `C:\Program Files\TrioKey` không còn | |
| 9m2 | Thoát. Đổi tên `%APPDATA%\triokey\config.txt` thành tên khác, chạy bản thường bằng "Run as administrator", mở `Cài đặt…`, đổi một ô | không có `config.txt` nào được tạo; cảnh báo "Không lưu được config.txt…" khi mở cửa sổ và sau khi đổi. Thoát, đổi tên file lại như cũ | |
| 9m3 | Chỉ khi có sẵn một máy không tách token: Windows Sandbox (có lẽ, chưa kiểm), tài khoản `Administrator` dựng sẵn, hoặc UAC tắt hẳn (`EnableLUA = 0`; kéo thanh trượt UAC xuống thấp nhất **không** đủ, token vẫn bị tách). Đừng tắt UAC trên máy chính chỉ để thử. Chạy TrioKey lần đầu ở đó, mở `Cài đặt…`, đổi một ô, chọn `Chế độ phát triển`, bấm `Mở thư mục cấu hình…` | `%APPDATA%\triokey\config.txt` được tạo và đổi theo; **không** có cảnh báo "Không lưu được config.txt…"; Explorer mở thư mục (ở đó không có phía quyền thường nào nên TrioKey xử sự như bản thường) | |
| 9n | (Từ bản thường) bật chế độ admin, bấm **No** ở câu "chạy ngay"; xem ô `Khởi động cùng Windows`; rồi bỏ chọn `Chạy quyền admin…` ngay trong bản thường | sau khi bấm No: `Khởi động cùng Windows` được chọn và mờ đi. Khi bỏ chọn: một UAC; thông báo đã tắt và xoá `C:\Program Files\TrioKey`; thư mục mất ngay; nếu `Khởi động cùng Windows` đã bật trước đó thì giá trị Run `TrioKey` trở lại | |

Chưa ai kiểm trên máy thật: `schtasks /Run` từ bản thường có chạy được tác vụ quyền cao nhất không (9g), schtasks có nhận file XML UTF-16 không (9c), phím "che" sau Alt+Z có chặn được thanh menu không (2e), biểu tượng có trở lại sau khi Explorer khởi động lại không (1f, 9j2), cửa sổ cài đặt trông ra sao với manifest mới (2a, 2k), biểu tượng của cửa sổ ở từng tỉ lệ (2m), `Khởi động lại` ở bản thường và bản admin (1h, 9j5) và giao diện tiếng Anh (2y đến 2y5). Nếu 9c báo "Không bật được chế độ admin", ghi lại. Chế độ admin giả định bạn là admin của máy và đồng ý UAC bằng chính tài khoản mình.

# Tự cập nhật (AT10)

Cần khoá ký thật trong repo (đã có) và ít nhất một bản đã phát hành trên GitHub.

**Chuẩn bị một bản "cũ":** sửa tạm `version` trong `crates/triokey/Cargo.toml` xuống thấp hơn bản đã phát hành (ví dụ `0.0.9`, **không commit**), build `mise x -- cargo build -p triokey --release --target-dir target/m4`, rồi trả `Cargo.toml` và `Cargo.lock` lại như cũ (`git checkout -- crates/triokey/Cargo.toml Cargo.lock`). Chép `target\m4\release\triokey.exe` vào một thư mục thử riêng (ví dụ `D:\tmp\tk`) và chạy từ đó. Trước mỗi bước, xoá dòng `last_update_check` trong `config.txt` khi TrioKey đang tắt, để nó kiểm tra lại ngay.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 10a | Bản build bình thường (cùng phiên bản với bản phát hành mới nhất), mở `Cài đặt…`, bấm `Kiểm tra bản mới` | dòng kết quả: `Đang kiểm tra…` rồi `Đang dùng bản mới nhất.`; cửa sổ không đơ trong lúc chờ và không đổi kích thước; `config.txt` có `last_update_check` khác 0 | |
| 10b | Chạy bản "cũ" từ thư mục thử, chờ khoảng một phút (không bấm gì) | thông báo `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau.`; trong thư mục có `triokey.exe` (mới: SHA-256 khớp `SHA256SUMS.txt` của bản phát hành) và `triokey.old.exe`, không còn `triokey.new.exe`; gõ tiếng Việt vẫn bình thường trong lúc tải và sau đó | |
| 10c | Ngay sau 10b: mở `Cài đặt…`, bấm `Kiểm tra bản mới`; nhấp phải khay | dòng kết quả `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau.`; menu khay **không** có dòng cập nhật | |
| 10d | `Thoát`, chạy lại `triokey.exe` trong thư mục thử, mở `Cài đặt…` | góc dưới ghi phiên bản mới; `triokey.old.exe` đã mất; nếu `Khởi động cùng Windows` đang bật thì vẫn bật | |
| 10e | Bản "cũ" lần nữa (chép lại), bỏ chọn `Tự động cập nhật`, `Thoát`, xoá dòng `last_update_check`, chạy lại và để 3 phút. Mở Resource Monitor → Network, lọc `triokey.exe` | không có kết nối mạng nào, không có thông báo, exe không đổi; `config.txt` có `auto_update = false`. Bấm `Kiểm tra bản mới` thì vẫn kiểm tra (và ở bản thường thì cài) | |
| 10f | Ngắt mạng (tắt Wi-Fi), bấm `Kiểm tra bản mới` | sau tối đa vài chục giây: `Chưa kiểm tra được: không tải được từ GitHub.` trên một dòng, cửa sổ không đổi cỡ (cả lúc hiện `Đang kiểm tra…`); gõ tiếng Việt không bị ảnh hưởng; `last_update_check` không đổi | |
| 10g | Chép bản "cũ" vào một thư mục tài khoản thường không ghi được (tạo bằng quyền admin, ví dụ `C:\Program Files\tk-test`), chạy **không** nâng quyền | thông báo `Có bản mới X.Y.Z`; menu khay có `Có bản X.Y.Z — mở trang tải về`; bấm thì trình duyệt mở trang phát hành; trong `Cài đặt…` dòng kết quả ghi `Có bản mới vX.Y.Z.` rồi liên kết `Cập nhật tại đây` trên cùng dòng, không có địa chỉ viết ra; bấm liên kết (hoặc Tab tới rồi phím cách / Enter) cũng mở trang phát hành; thư mục không có file mới nào. Dọn: xoá thư mục thử bằng quyền admin | |
| 10h | Từ bản "cũ" trong thư mục thử: bật `Chạy quyền admin…`, chọn chạy bản admin ngay, chờ khoảng một phút | thông báo `Có bản X.Y.Z — bấm để cập nhật`; `C:\Program Files\TrioKey\triokey.exe` **chưa** đổi và thư mục đó không có file mới nào (bản admin không tự cài; nó tải file exe vào bộ nhớ để kiểm chữ ký trước khi báo); `last_update_check` trong `config.txt` **không** đổi (bản admin không tự ghi file này); menu khay có dòng `Có bản X.Y.Z — bấm để cập nhật`; dòng kết quả trong `Cài đặt…` ghi `Có bản mới vX.Y.Z.` rồi liên kết `Cập nhật tại đây` | |
| 10i | Bấm vào dòng đó trong menu khay (hoặc vào thông báo, hoặc liên kết `Cập nhật tại đây` trong `Cài đặt…`) | không có hộp UAC; thông báo `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau.`; trong `C:\Program Files\TrioKey` có exe mới và `triokey.old.exe`; `triokey.exe` trong thư mục thử (bản thường) **không** bị đụng tới. Đăng xuất/vào lại: bản admin là phiên bản mới, `triokey.old.exe` đã mất | |
| 10j | Tắt chế độ admin. Nhấp phải bản "cũ" trong thư mục thử → Run as administrator | thông báo `Có bản mới X.Y.Z`; bấm vào, hoặc bấm liên kết `Cập nhật tại đây` trong `Cài đặt…`, thì hiện hộp thoại ghi địa chỉ trang phát hành (trình duyệt **không** tự mở); thư mục thử không có `triokey.new.exe` hay `triokey.old.exe`, exe không đổi | |
| 10k | Trong lúc 10b đang tải (hoặc ngay sau), bấm `Chép thông tin debug` trong `Cài đặt…` (chọn `Chế độ phát triển` trước) và đọc dòng `hook:` | p99 "không gửi" vẫn dưới 1000 µs | |

Chưa ai kiểm trên máy thật: WinHTTP có đi qua được hai lần chuyển hướng của GitHub không, proxy của máy có được dùng không, thông báo kiểu "balloon" hiện ra sao trên Windows 11 và bấm vào có tới TrioKey không, và việc đổi tên file exe đang chạy trong thư mục thường lẫn trong `Program Files`. Cũng chưa thử: thay tên `triokey.exe` trong một bước khi nó đang chạy (`ReplaceFileW`, hoặc hard link rồi đổi tên đè) có được không; nếu được thì bỏ được khoảng hở giữa hai lần đổi tên ghi ở `docs/ARCHITECTURE.md`, 9.5.

# Ghi nhớ từ đã đổi (AT12)

Đổi một từ bằng một lần Backspace rồi gõ phím cách, Tab hoặc Enter thì TrioKey nhớ lần đổi đó trong `%APPDATA%\triokey\learned.txt` (mỗi dòng `phím gõ<TAB>chữ hiện ra`) và dùng lại từ lần sau. File được ghi khoảng 2 giây sau thay đổi cuối, khi Thoát và khi đăng xuất/tắt máy. Phần engine đã được kiểm bằng test; việc ghi file trong bản admin mới chỉ được kiểm trên thư mục tạm.

Notepad, bản thường (chưa bật chế độ admin), mỗi bước ở một dòng mới. Xem `learned.txt` bằng một cửa sổ Notepad khác, **không lưu** file đó khi TrioKey đang chạy (TrioKey sẽ ghi đè).

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| 12a | Trước khi gõ: mở `learned.txt` | có sẵn, chỉ có hai dòng `#` | |
| 12b | `this ` rồi ⌫ rồi `,` | `thí,`; `learned.txt` không có dòng `this` (dấu câu ngay sau lần đổi: không nhớ) | |
| 12c | `this ` rồi ⌫ rồi phím cách; chờ 3 giây, mở lại `learned.txt` | `thí `; file có dòng `this` + Tab + `thí` | |
| 12d | Dòng mới: `this This ` | `thí Thí ` (từ đã nhớ, chữ hoa theo phím gõ) | |
| 12e | Dòng mới: `this ` rồi ⌫ rồi phím cách; chờ 3 giây, mở lại `learned.txt`; dòng mới `this ` | `thí ` → `this `; dòng `this` đã mất khỏi file; lần gõ sau ra `this ` | |
| 12f | Làm lại 12c. `Thoát`, mở `learned.txt`, chạy lại TrioKey, gõ `this ` | file còn dòng `this` + `thí`; sau khi chạy lại vẫn ra `thí `. Dọn: làm 12e | |
| 12g | `Thoát`. Thêm vào `learned.txt` một dòng `this` + Tab + `thí`, lưu, chạy lại, gõ `this `; `Thoát`, xoá dòng đó, lưu, chạy lại, gõ `this ` | lần đầu `thí `, lần sau `this ` | |
| 12h | Bản thường đang chạy: đổi tên `learned.txt` thành tên khác, rồi `Cài đặt…` → chọn `Chạy quyền admin…`, khi UAC hiện thì xem thư mục, bấm **No** | `learned.txt` mới (chỉ có hai dòng `#`) đã có trước khi UAC hiện; bấm No không đổi gì khác. Xoá file mới, đổi tên file cũ lại | |
| 12i | Bật lại chế độ admin và chạy bản admin (AT9, bước 9c và 9g): làm 12c trong Notepad thường | `thí `; `learned.txt` có dòng `this` + `thí` (bản admin ghi được file có sẵn). Dọn: làm 12e | |
| 12j | Bản admin đang chạy: đổi tên `learned.txt` thành tên khác; đăng xuất, đăng nhập lại (bản admin tự chạy); làm 12c rồi chờ 3 giây; `Thoát` | trong phiên đó vẫn ra `thí ` lần sau (nhớ trong bộ nhớ); **không** có `learned.txt` nào được tạo, kể cả sau khi Thoát. Đổi tên file cũ lại | |

# macOS (bản thử nghiệm, M1–M37)

Bản macOS được viết trên Windows. M21–M32 kiểm cửa sổ cài đặt, file, phím tắt, khởi động cùng máy, kiểm tra bản mới và thông tin debug; M33–M37 kiểm biểu tượng màu trên menu bar, biểu tượng trong menu, khởi động lại, ngôn ngữ giao diện và gõ nhanh `maw`. Cột `Mong đợi` là điều mã nguồn định làm, không phải điều đã thấy. Thiết kế: `docs/ARCHITECTURE.md`, mục 12. Làm lần lượt từ trên xuống, ghi kết quả vào ô `Kết quả`, và ghi phiên bản macOS (menu Apple → Giới thiệu về máy Mac này) cùng loại máy (Apple silicon hay Intel).

**Trước khi bắt đầu**

1. Tải `TrioKey-X.Y.Z-macos.dmg` ở trang Releases của repo (bản phát hành mới nhất, từ 0.3.0). Kiểm tra: `shasum -a 256 TrioKey-X.Y.Z-macos.dmg` phải khớp dòng của nó trong `SHA256SUMS.txt` cùng bản phát hành. (Bản dựng giữa hai lần phát hành: trang Actions, workflow `macOS`, artifact `TrioKey-macos-dmg`, cần đăng nhập GitHub; ghi lại lần chạy đó có bước `Self-test` xanh hay đỏ, vì dmg vẫn được tải lên cả khi self-test trượt.)
2. Thoát mọi bộ gõ tiếng Việt khác. Nếu đang dùng nguồn nhập tiếng Việt có sẵn của macOS thì chuyển sang ABC hoặc U.S.: TrioKey chỉ gõ khi nguồn nhập là một bàn phím Latin thường (ABC, U.S.…); với mọi bộ gõ khác (kể cả tiếng Nhật ở chế độ Latin) nó nhường.
3. Menu bar: chữ `V` đỏ = đang gõ tiếng Việt, chữ `E` xanh = mọi trường hợp khác. Di chuột lên chữ để xem lý do (tooltip có thể không hiện với mục menu bar; khi đó dùng `Chép thông tin debug` (M7), dòng `state:` cho biết).

**Thoát:** bấm chữ `V`/`E` trên menu bar → `Thoát`. Nếu bàn phím loạn mà không bấm được menu: Activity Monitor → chọn TrioKey → nút ⓧ → Buộc thoát. **Không** tắt hay xoá TrioKey trong Trợ năng (Accessibility) khi nó đang chạy: theo các báo cáo của lập trình viên, việc đó có thể làm bàn phím và chuột đứng tới khi khởi động lại máy.

| Bước | Thao tác | Mong đợi | Kết quả |
| --- | --- | --- | --- |
| M1 | Mở dmg, kéo `TrioKey` vào lối tắt `Applications` bên cạnh. Mở file `DOC TRUOC - READ FIRST.txt` | TrioKey nằm trong thư mục Applications; file đọc trước có hai phần tiếng Việt và tiếng Anh | |
| M2 | Mở TrioKey **từ Applications** (không phải từ dmg) | macOS báo không mở được vì chưa kiểm được nhà phát triển; bấm Xong. Ghi lại đúng chữ trong hộp thoại | |
| M3 | Cài đặt hệ thống → Quyền riêng tư & Bảo mật, kéo xuống, bấm `Vẫn mở` (Open Anyway), rồi `Mở` ở hộp thoại sau. Ghi lại nếu các bước khác đi | TrioKey chạy: không có biểu tượng ở Dock, chữ `E` hiện trên menu bar | |
| M4 | Ngay sau M3 | Cửa sổ `Hướng dẫn cấp quyền` hiện ra phía trước, dòng trạng thái in đậm `Chưa có quyền`; có thể kèm hộp thoại của macOS mời mở cài đặt Trợ năng. Menu bar `E`, tooltip `TrioKey: chưa có quyền Accessibility` | |
| M5 | Bấm nút `Mở cài đặt Accessibility` | Cài đặt hệ thống mở đúng trang Quyền riêng tư & Bảo mật → Trợ năng. Nếu mở trang khác hoặc không mở: ghi lại rồi tự vào trang đó | |
| M6 | Bật công tắc TrioKey (không thấy trong danh sách thì bấm `+` và chọn TrioKey trong Applications). Nhìn cửa sổ hướng dẫn, không khởi động lại TrioKey | Trong vòng khoảng 2 giây: dòng trạng thái đổi thành `Đã có quyền, TrioKey đang chạy`, menu bar thành `V` (nguồn nhập ABC/U.S.). Nếu sau 10 giây vẫn `Chưa có quyền`: Thoát, mở lại TrioKey từ Applications, ghi lại là phải mở lại | |
| M7 | Menu → `Cài đặt…` → tích `Chế độ phát triển` → `Chép thông tin debug`, dán vào ô Kết quả hoặc gửi lại | Một hộp thoại báo đã chép; đoạn chữ có `tap: created yes, enabled yes`, `bundle: /Applications/TrioKey.app (AppTranslocation: no)`, `state: TrioKey: Tiếng Việt`, và không chứa chữ nào bạn đã gõ | |
| M8 | TextEdit (một văn bản mới), mỗi bước một dòng: `tieesng vieejt ` · `DDuowngf ` · `windows terrible ` | `tiếng việt ` · `Đường ` · `windows terrible ` | |
| M9 | TextEdit, dòng trộn: `tooi caanf test laij ` · `OS naof cungx dduwowcj ` | `tôi cần test lại ` · `OS nào cũng được ` | |
| M10 | TextEdit, Backspace: `this ` rồi ⌫ · dòng mới `this ` rồi ⌫ ⌫ · dòng mới `tooi ` rồi ⌫ | `thí` · `th` · `tôi` (chỉ xoá dấu cách) | |
| M11 | TextEdit: gõ `w` (thấy `ư`) rồi Enter · dòng mới `w` rồi Tab | `w` rồi xuống dòng thật · `w` rồi một ký tự Tab. Trong một ứng dụng chat mà Enter là gửi (Messages, Slack…): `w` rồi Shift+Enter thì xuống dòng, không gửi | |
| M12 | Trang web trong Safari hoặc Chrome (một ô soạn thảo nhiều dòng, không phải thanh địa chỉ): làm lại M8 và M10 | như M8 và M10 | |
| M13 | Thanh địa chỉ Safari hoặc Chrome: gõ `tieesng vieejt` (xoá trắng ô trước) 5 lần | ghi lại, không chặn: ô có tự điền có thể làm thiếu hoặc thừa một ký tự | / 5 |
| M14 | Terminal (và iTerm2 nếu có): `echo tieesng vieejt ` | ghi lại, không chặn. Nếu Terminal đang bật `Secure Keyboard Entry` thì TrioKey không gõ được (xem M17) | |
| M15 | TextEdit: gõ `tiee`, rồi Command+A, Command+C, Command+V, Command+Z; gõ `tiee`, mũi tên trái, gõ `s`; Option+Delete | các phím tắt chạy như khi không có TrioKey, không có chữ thừa; sau mũi tên `s` chỉ chèn chữ `s`; Option+Delete xoá cả từ | |
| M16 | Thêm nguồn nhập tiếng Nhật (Cài đặt → Bàn phím → Nguồn nhập). Chuyển sang Hiragana từ menu nguồn nhập: gõ `nihongo` rồi Enter. Chuyển sang chế độ Latin của bộ gõ Nhật (英字), gõ `tieesng `. Chuyển lại ABC, gõ `tieesng `. Mỗi lần chuyển, nhìn chữ V/E. Cuối cùng `Chép thông tin debug` (như M7), dán đoạn chữ vào ô Kết quả | Hiragana: `E` (tooltip `TrioKey: Tiếng Anh — đang nhường bộ gõ khác`), bộ gõ Nhật hoạt động như thường (日本語). Chế độ Latin của bộ gõ Nhật: cũng `E`, ra `tieesng ` (cố ý: bản này nhường mọi bộ gõ). ABC: `V` trong vòng 2 giây, ra `tiếng `. Phần `input sources seen since launch` liệt kê các nguồn đã chuyển qua | |
| M17 | Secure Input: Terminal → menu Terminal → bật `Secure Keyboard Entry`, rồi đưa Terminal lên trước; sau đó bấm vào một ô mật khẩu của Safari (trang đăng nhập bất kỳ). Tắt lại Secure Keyboard Entry | Khi Secure Keyboard Entry bật và Terminal ở trước: `E` trong vòng 2 giây, tooltip `TrioKey: bàn phím đang bị Secure Input chặn`; tắt thì lại `V`. Ô mật khẩu Safari: ghi lại là `E` (Secure Input) hay vẫn `V` | |
| M18 | Menu → bỏ dấu tick `Tiếng Việt`, gõ `tieesng `; bấm lại | đã tắt: `E`, tooltip `TrioKey: Tiếng Anh — đã tắt`, ra `tieesng `; bật lại: `V`, ra `tiếng ` | |
| M19 | Khi TrioKey đang chạy, trong Terminal: `open -n /Applications/TrioKey.app` (mở thêm một bản; chưa chắc `open -n` là cách đúng để thử, Finder thường chỉ đưa bản đang chạy lên trước). Sau đó menu → `Thoát` | bản thứ hai tự thoát ngay: menu bar chỉ có một chữ `V`/`E`, Activity Monitor chỉ có một TrioKey. Sau Thoát: chữ trên menu bar biến mất, bàn phím và chuột vẫn bình thường, gõ `tieesng ` ra `tieesng ` | |
| M20 | Cài bản mới (khi có bản dựng sau), đúng thứ tự: (1) `Thoát` TrioKey; (2) kéo TrioKey mới vào Applications, chọn Thay thế; (3) Cài đặt → Quyền riêng tư & Bảo mật → Trợ năng: chọn TrioKey, bấm `−`, rồi `+` và chọn lại TrioKey trong Applications; (4) mở TrioKey (có thể phải làm lại M2–M3). **Không bao giờ làm bước (3) khi TrioKey đang chạy** | `V`; thông tin debug (M7) có số phiên bản mới. Mỗi bản dựng được ký ad hoc nên quyền cũ không dùng được cho bản mới; ghi lại nếu bỏ bước (3) thì có còn quyền hay không | |
| M21 | Menu → `Cài đặt…`; bấm lại `Cài đặt…` khi cửa sổ đang mở; đổi Giao diện của macOS giữa Sáng và Tối | một cửa sổ `TrioKey – Cài đặt` (lần hai chỉ đưa nó lên trước), không kéo giãn được; thứ tự: `Bật gõ tiếng Việt` … `Phím tắt`, `Gõ`, `Hệ thống`, `Cập nhật`, đường kẻ, `TrioKey v<phiên bản>` bên trái và `GitHub` bên phải; đọc rõ ở cả Sáng lẫn Tối, không chữ nào bị cắt. Chụp ảnh cả hai | |
| M22 | Tích / bỏ `Chế độ phát triển` | chỉ lúc này cửa sổ đổi chiều cao; khi bật có hai nút `Mở thư mục cấu hình…` (mở Finder ở `~/Library/Application Support/triokey`, có `config.txt` và `learned.txt`) và `Chép thông tin debug` | |
| M23 | `Kiểm tra bản mới` (có mạng), rồi tắt Wi-Fi và bấm lại | dòng dưới lần lượt `Đang kiểm tra…` rồi `Đang dùng bản mới nhất.` (hoặc `Có bản mới v….` kèm link `Tải tại đây` mở trang Releases); không mạng: `Chưa kiểm tra được: không tải được từ GitHub.`. Cửa sổ không đổi kích thước lần nào | |
| M24 | Đổi kiểu bỏ dấu sang `hoà, thuý`, gõ `hoaf `; bỏ `Tự giữ nguyên từ tiếng Anh`, gõ `windows `; Thoát, mở lại, mở `Cài đặt…` | `hoà ` ngay (không cần mở lại); `windows` thành chữ Telex; sau khi mở lại các ô giữ nguyên lựa chọn. Bật lại hai ô như cũ | |
| M25 | Phím tắt `Control+Shift`: bấm giữ Control, bấm Shift, thả cả hai. Rồi Control+Shift+Tab trong Safari | lần đầu đổi `V`↔`E`; phím tắt Control+Shift+Tab của Safari chạy bình thường và không đổi `V`/`E` | |
| M26b | Chọn `Option+Shift`: bấm giữ Option, bấm Shift, thả cả hai; rồi Option+Shift+K trong TextEdit | lần đầu đổi `V`↔`E`; Option+Shift+K vẫn ra ký tự của nó và không đổi `V`/`E` | |
| M26 | Chọn `Option+Z` trong `Phím tắt`; trong TextEdit bấm Option+Z, rồi giữ Option+Z một lúc; chọn `Không dùng` rồi bấm Option+Z | Option+Z đổi `V`↔`E` một lần, không chèn `Ω`; giữ lâu cũng chỉ đổi một lần. `Không dùng`: Option+Z ra `Ω`. Chọn lại `Control+Shift` | |
| M27 | Bỏ `Nhường khi bộ gõ Nhật/Trung/Hàn đang bật`, chọn nguồn nhập Hiragana, gõ `tieesng `. Bật lại ô đó | khi bỏ: `V` cả khi đang Hiragana, ra `tiếng ` (bộ gõ Nhật nhận chữ đã sửa); bật lại: `E` | |
| M28 | Chạy TrioKey từ Downloads (không phải Applications), mở `Cài đặt…` | `Khởi động cùng máy` bị mờ, bên cạnh ghi `Chỉ dùng được khi TrioKey nằm trong Applications` | |
| M29 | Chạy từ Applications, tích `Khởi động cùng máy`, đăng xuất rồi đăng nhập lại; rồi khởi động lại máy; sau đó bỏ tích. (Chưa ai biết bản do launchd chạy lúc đăng nhập có giữ được quyền Trợ năng không: dòng này kiểm đúng điều đó) | có file `~/Library/LaunchAgents/io.github.ved0el.triokey.plist`; macOS có thể báo "Background Items Added"; sau khi đăng nhập lại và sau khi khởi động lại máy, TrioKey tự chạy và vẫn gõ được (quyền Trợ năng còn dùng được). Ghi lại thông báo của macOS. Bỏ tích: file biến mất. Menu → `Thoát` vẫn thoát hẳn (không tự chạy lại) | |
| M30 | Gõ `this ` rồi ⌫ (giữ `thí`), đợi 3 giây, Thoát, mở lại, gõ `this ` | `learned.txt` có dòng `this` `thí`; sau khi mở lại `this ` ra `thí ` | |
| M31 | Thoát. Lưu `config.txt` thành UTF-16 (TextEdit → Lưu dưới dạng, Mã hoá văn bản UTF-16), mở TrioKey, mở `Cài đặt…`, đổi một ô, Thoát. Rồi xoá `config.txt` | một hộp thoại `Không đọc được config.txt…` lúc mở cửa sổ (một lần); file UTF-16 không bị ghi đè. Sau khi xoá, lần chạy sau tạo lại file mặc định | |
| M32 | Thông tin debug (M7) sau M23–M30 | có `config folder: ~/Library/Application Support/triokey`, dòng `settings:` với mọi khoá (có `language = vi`), `remembered words:` là một con số, `start at login:`, `update:`; không có tên người dùng, không có chữ đã gõ | |
| M33 | Biểu tượng menu bar: Cài đặt hệ thống → Giao diện → Sáng, nhìn biểu tượng khi đang `V` và khi tắt (`E`); đổi sang Tối khi TrioKey đang chạy, nhìn lại; thử thêm với một hình nền sẫm màu khi đang ở Sáng. Chụp ảnh menu bar mỗi lần | chữ `V`/`E` đậm, có màu như khay Windows (V đỏ, E xanh), không phải chữ mảnh; Sáng: màu đậm, Tối: màu nhạt, đọc rõ trên nền thanh menu; đổi Sáng/Tối thì màu đổi theo trong vòng khoảng 2 giây, không cần mở lại. Nét sắc trên màn hình Retina và màn hình thường (nếu có). Ghi lại nếu màu không hợp với nền (vd. hình nền sẫm ở chế độ Sáng) | |
| M34 | Bấm biểu tượng menu bar, nhìn menu; khi có bản mới (M23) nhìn lại | `Tiếng Việt` có dấu tick, không có hình; `Cài đặt…` (bánh răng), `Hướng dẫn cấp quyền…` (khiên có khoá), đường kẻ, `Khởi động lại` (mũi tên vòng), `Thoát` (nút nguồn); không còn dòng số phiên bản. Khi có bản mới: `Có bản … — mở trang tải về` có hình mũi tên xuống trong vòng tròn | |
| M35 | Chạy từ Applications, menu → `Khởi động lại`; đợi tới khi biểu tượng hiện lại, mở Activity Monitor, gõ `tieesng ` trong TextEdit. Làm lại 3 lần | biểu tượng biến mất rồi hiện lại trong vòng vài giây; Activity Monitor chỉ có một TrioKey; gõ ra `tiếng ` (quyền Trợ năng vẫn dùng được, không phải cấp lại); các lựa chọn trong `Cài đặt…` giữ nguyên. Lần nào TrioKey không tự mở lại: ghi lại và mở lại bằng tay | |
| M36 | `Cài đặt…` → `Ngôn ngữ` → `English`, nhìn cửa sổ ngay lúc đó; mở menu, di chuột lên biểu tượng, mở `Permission guide…`; `Copy debug info` (developer mode); Thoát, mở lại; rồi chọn lại `Tiếng Việt` | ngay khi chọn: cửa sổ cài đặt toàn tiếng Anh (`TrioKey – Settings`, `Enable Vietnamese`, `Shortcut`, `None`, `Typing`, `System`, `Language`, `Updates`…), đứng nguyên chỗ cũ, rộng ra nếu cần, không chữ nào bị cắt; menu `Vietnamese`, `Settings…`, `Permission guide…`, `Restart`, `Quit`; tooltip `TrioKey: Vietnamese`; cửa sổ hướng dẫn tiếng Anh, lời cảnh báo (QUIT TrioKey first) vẫn đỏ và đủ chữ; hộp thoại chép debug tiếng Anh (đoạn chữ debug giữ như cũ). Sau khi mở lại vẫn tiếng Anh. Chọn `Tiếng Việt`: mọi thứ trở lại tiếng Việt ngay. Chụp ảnh cửa sổ cài đặt tiếng Anh ở Sáng và Tối | |
| M37 | TextEdit, gõ thật nhanh (không thả phím trước khi bấm phím sau), mỗi từ 5 lần: `maw ` · `tawts ` · `suawr ` | `mă ` · `tắt ` · `sửa ` lần nào cũng đủ chữ (lỗi cũ: ra `m`, `ắt`, `ửa`); làm lại trong Safari/Chrome và một ứng dụng Electron nếu có | / 15 |

**Gỡ:** bỏ tích `Khởi động cùng máy` (hoặc xoá `~/Library/LaunchAgents/io.github.ved0el.triokey.plist`); `Thoát`; trong Trợ năng xoá TrioKey (`−`) sau khi đã thoát; xoá `TrioKey` khỏi Applications; xoá `~/Library/Application Support/triokey` nếu muốn bỏ cấu hình và các từ đã nhớ. TrioKey không ghi file nào khác.

# Gỡ hẳn

1. Thoát TrioKey (nhấp phải khay → `Thoát`).
2. Nếu đã bật chế độ admin, PowerShell **quyền admin**: `schtasks /Delete /TN TrioKey /F` rồi `Remove-Item -Recurse "C:\Program Files\TrioKey"`.
3. `reg delete HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v TrioKey /f` (báo không tìm thấy cũng được).
4. Xoá `%APPDATA%\triokey` nếu muốn bỏ cấu hình và các từ đã nhớ (`learned.txt`).
5. Xoá thư mục chứa `triokey.exe`. Sau một lần tự cập nhật, trong đó có thể còn `triokey.old.exe` (bản trước); xoá cùng luôn.

TrioKey không ghi gì khác: không layout bàn phím, không danh sách ngôn ngữ, không mục nào trong `HKCU\Keyboard Layout`.
