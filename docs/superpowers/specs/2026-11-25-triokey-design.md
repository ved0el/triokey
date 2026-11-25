# TrioKey — bộ gõ tiếng Việt cho người gõ Anh / Việt / CJK (Chinese, Japanese, Korean)

- Ngày: 2026-11-25
- Trạng thái: người dùng đã duyệt ngày 2026-11-25 kèm hai điều chỉnh, đã đưa vào bản này: (1) cần gõ được trong cửa sổ admin; (2) phần bỏ dấu dựa trên cơ chế đã tối ưu của các bộ gõ hiện có, không tự chế. Người dùng giao: agent Opus code và test, chỉ dừng khi cần gõ thử trực tiếp, gặp lựa chọn thì theo hướng tốt nhất.
- Tên: TrioKey (repo https://github.com/ved0el/triokey). Tên tạm lúc viết spec là EVJ.
- Phạm vi: CJK (Chinese, Japanese, Korean). Tiếng Nhật đã được gõ thử tay (AT1–AT3). Tiếng Trung và tiếng Hàn có luật được kiểm bằng unit test nhưng chưa gõ thử tay; các bước nằm ở mục AT2-CK của `docs/HANDS_ON_TEST.md`.

## 1. Vì sao làm

Người dùng gõ lẫn tiếng Anh, tiếng Việt (Telex) và tiếng Nhật (Microsoft IME) mỗi ngày, hiện dùng VKey. Hai chỗ vướng do chính người dùng nêu:

1. Chưa tự chuyển Anh/Việt được.
2. Đang gõ Việt mà sang Nhật thì phải bật hiragana, rồi còn phải tắt chế độ V của bộ gõ mới gõ được.

Người dùng cũng nêu herdr và ssh nhận chữ không đúng, rồi quyết định bỏ qua hai trường hợp này (mục 2.2).

Mục tiêu trong một câu: gõ được cả ba thứ tiếng mà không bao giờ phải bật/tắt bộ gõ. Bật/tắt Microsoft IME không tính.

## 2. Phạm vi

### 2.1 Làm

| # | Mục tiêu | Xuất phát từ |
| --- | --- | --- |
| G1 | Tự nhường khi IME CJK đang soạn chữ; gõ Việt bình thường khi IME ở direct input | yêu cầu gốc + chỗ vướng 2 |
| G2 | Gõ Telex, từ tiếng Anh tự được giữ nguyên; không có chế độ V/E | yêu cầu gốc + chỗ vướng 1 |
| G3 | Không thêm, không đổi layout / ngôn ngữ / danh sách bàn phím của Windows | yêu cầu gốc |
| G4 | Gõ đúng trong ô có tự điền: thanh địa chỉ trình duyệt là bắt buộc, ô Excel là cố gắng | nhóm lỗi "xoá lùi + chèn chữ không êm" đã trao đổi; cần cho việc dùng Brave hằng ngày |
| G5 | Cửa sổ cài đặt chỉ gồm thứ cần dùng | yêu cầu gốc; "gõ tắt, nhớ theo app không cần" |
| G6 | Viết bằng Rust; lõi gõ tách khỏi Windows để sau này dùng lại cho macOS/Linux | yêu cầu gốc |
| G7 | Gõ được tiếng Việt trong cửa sổ chạy quyền admin | người dùng yêu cầu khi duyệt |
| G8 | Tự cập nhật bằng bản phát hành có chữ ký trên GitHub, tắt được | người dùng yêu cầu sau M4 |

### 2.2 Chưa làm, và điều gì sẽ kéo nó vào

| Việc | Điều kiện kéo vào |
| --- | --- |
| Cách gửi chữ riêng cho terminal (herdr, ssh), kể cả ô xem trước | người dùng đã nói bỏ qua; kéo vào khi họ muốn quay lại |
| VNI, VIQR, bảng mã cũ (TCVN3, VNI-Windows) | người dùng cần |
| Gõ tắt, nhớ chế độ theo ứng dụng | người dùng đã nói không cần |
| Emoji, âm báo, công cụ chuyển mã, nhật ký gõ | người dùng yêu cầu |
| Vỏ macOS / Linux | bản Windows đã dùng hằng ngày ổn định |
| Kiểm thử tiếng Trung / Hàn | người dùng bắt đầu gõ hai thứ tiếng đó |
| Thống kê cặp từ, nhìn từ đứng sau để sửa lại từ đứng trước | sau M3 mà từ trùng vẫn sai quá mức chịu được |
| Học từ thói quen sửa tay của người dùng | cùng một từ bị đổi tay lặp đi lặp lại |
| Nhận biết ô mật khẩu trong trình duyệt / Electron | bị sai mật khẩu vì bộ gõ dù chỉ một lần |
| Giao diện cho từ điển riêng | file văn bản không còn đủ |

### 2.3 Không bao giờ

- Đăng ký TSF, cài layout, thêm ngôn ngữ, đổi layout mặc định của người dùng.
- Ghi phím hoặc nội dung gõ ra đĩa.
- Dùng mạng cho việc gì khác ngoài kiểm tra và tải bản cập nhật có chữ ký từ GitHub (5.8), hoặc gửi đi nội dung gõ hay bất kỳ mã nhận dạng nào. Trước M5 dòng này là "không mở kết nối mạng"; nó được thu hẹp lại theo yêu cầu của người dùng (G8), và khi tắt việc kiểm tra cập nhật thì TrioKey vẫn hoàn toàn không dùng mạng.
- Làm phím gõ chậm tới mức cảm nhận được.

## 3. Dữ kiện

### 3.1 Máy và thói quen hiện tại (25/11/2026)

- Windows 11. Hai layout: `0409` (en-US, đứng đầu danh sách) và `0411` (Nhật, Microsoft IME). Bàn phím vật lý kiểu US 101 phím.
- Layout khác nhau theo từng cửa sổ: cùng lúc Brave ở `0411`, các cửa sổ khác ở `0409`.
- Cấu hình VKey đang dùng: Telex, bỏ dấu kiểu cũ (`hòa`), gõ tắt tắt, nhớ theo app tắt, "tự tắt khi bàn phím CJK" bật, không chạy quyền admin, phím chuyển V/E là Ctrl+Shift.
- Chưa có Rust, chưa có bộ build C++ (MSVC).

### 3.2 Thử nghiệm đọc trạng thái IME

Script chỉ đọc, hỏi cửa sổ IME mặc định của cửa sổ đang focus (`ImmGetDefaultIMEWnd` + `WM_IME_CONTROL`), lấy mẫu mỗi 100 ms. Lần chạy dừng ở khoảng giây 39.

| Ứng dụng | Kết quả |
| --- | --- |
| Brave (`Chrome_WidgetWin_1`) | `imeOpen` đổi 0 → 1 → 0 đúng lúc bật/tắt hiragana |
| Ô Search của Windows (`Windows.UI.Core.CoreWindow`) | như trên |

- Tín hiệu dùng được là **open status**. Conversion mode đứng yên ở 9 sau khi tắt hiragana, nên không dùng nó cho tiếng Nhật.
- Một lần hỏi thường mất 0,1 – 3 ms; có một lần mất 28 ms (lần hỏi đầu tiên sau khi Brave lên trước). Vì vậy đường xử lý phím không hỏi trực tiếp mà đọc giá trị đã lưu sẵn (ngoại lệ duy nhất ở 5.3).

### 3.3 Chưa kiểm chứng

- Gõ tiếng Việt bằng "xoá lùi + ký tự Unicode" khi cửa sổ đang ở layout `0411` và IME đóng. Chưa ai thử; đây là phép thử nghiệm thu đầu tiên (AT1).
- Đọc trạng thái IME trong terminal, Office và ứng dụng Electron.
- Bật/tắt hiragana rồi gõ ngay: lấy mẫu 100 ms không nói gì về độ trễ này (AT3).
- Vì sao hiện nay người dùng phải tắt V bằng tay dù VKey đang bật "tự tắt khi bàn phím CJK". Thiết kế không dựa vào lời giải thích nào cho việc này.

## 4. Các hướng đã cân nhắc

Người dùng giao quyền chọn kiến trúc. Các lựa chọn dưới đây là của người viết, đã nêu trong lúc trao đổi và chưa bị bác.

| Hướng | Được gì | Mất gì | Kết luận |
| --- | --- | --- | --- |
| **A. Tự viết bằng Rust: keyboard hook + gửi chữ + lõi gõ riêng** | Đúng G1, G3, G6; mã nguồn của riêng người dùng, không ràng buộc GPL | Tự chuyển Anh/Việt phải tự xây và tinh chỉnh dần | **Chọn** |
| B. Text service TSF (như VieLotus bản Windows) | Cách "chính thống", ứng dụng tự vẽ chữ đang soạn nên không cần xoá lùi | Phải đăng ký vào danh sách bàn phím (trái G3) và chỉ một bộ gõ TSF hoạt động tại một thời điểm, tức là Microsoft IME phải nhường chỗ (trái G1) | Loại |
| C. Fork Vikey (C++, GPL-3.0), thêm phần nhận biết IME | Nhanh nhất để có chất lượng tự chuyển Anh/Việt của Vikey | Không phải Rust, mang GPL-3.0, thừa kế code và tính năng không cần | Không chọn; là đường lui nếu A sa lầy ở M3 |
| D. Vỏ Rust + engine C++ của VieLotus (MIT) qua C API | Ít code mới | Cần thêm bộ build C++; phụ thuộc một engine lớn không tự kiểm soát | Không chọn; dùng bộ test MIT của họ làm thước đo |

**Phần bỏ dấu: dùng lại cơ chế đã tối ưu (điều chỉnh của người dùng).** Biến đổi Telex và vị trí dấu thanh không tự chế. Ứng viên là thư viện Rust `vi` 0.8.0 (MIT, github.com/ZeroX-DG/vi-rs): bản phát hành có `transform_buffer_with_style`, `AccentStyle::Old` (bỏ dấu kiểu cũ) và `IncrementalBuffer`, đúng mô hình "tính lại cả từ" ở 5.4. Thử nhanh 20 dãy phím trên máy này: bỏ dấu đúng cả hai kiểu, phím lặp cho kết quả Telex kinh điển; nó không trả lại phím cho từ không phải tiếng Việt (`windows` → `uindows`), việc đó thuộc về lớp của TrioKey.

Người dùng nói rõ: chỉ dùng nếu thư viện thực sự dùng được, không thì tự viết nếu tối ưu hơn cho kiến trúc này. Vì vậy việc đầu tiên của M1 là chạy bảng ca kiểm thử của 5.4 qua `vi` rồi quyết định theo thứ tự ưu tiên: dùng nguyên bản phát hành → ghim một commit → chép vào repo và sửa (MIT cho phép) → tự viết theo mô hình 5.4 nếu sửa tốn hơn viết mới.

Với hành vi quanh phím lặp và trả lại phím, làm theo các quy tắc đã công bố và đã đo của Vikey (nhật ký quyết định D17–D19). Từ VKey và Vikey (GPL-3.0) chỉ lấy hành vi được mô tả trong tài liệu, không lấy mã nguồn, bảng dữ liệu hay file test. `vi` và VieLotus (MIT) được phép đọc và chuyển thể, có ghi nguồn.

## 5. Thiết kế

### 5.1 Hai phần

- **`engine`**: thư viện Rust thuần, không gọi hệ điều hành. Nhận phím, trả về lệnh sửa chữ. Toàn bộ luật Telex và quyết định Anh/Việt nằm ở đây, có test tự động. Đây là phần mang sang macOS/Linux sau này. Không dựng thêm lớp trừu tượng nào cho các nền tảng chưa tồn tại.
- **`triokey.exe`**: một tiến trình chạy nền với biểu tượng ở khay hệ thống. Bắt phím bằng `WH_KEYBOARD_LL`, hỏi `engine`, gửi chữ bằng `SendInput`. Không cài đặt gì vào Windows ngoài (tuỳ chọn) một mục tự khởi động.

### 5.2 Khi nào gõ gì

| Trạng thái của cửa sổ đang gõ | TrioKey làm gì | Biểu tượng khay |
| --- | --- | --- |
| Layout CJK và IME đang mở (hiragana, katakana…) | Cho mọi phím đi thẳng tới IME, không đụng vào | あ |
| Layout CJK và IME đóng (direct input) | Gõ Telex, tự giữ nguyên từ tiếng Anh | V |
| Layout không phải CJK (ENG…) | Như dòng trên | V |
| Người dùng tắt tay bằng phím tắt | Cho mọi phím đi thẳng | – |

"Chỉ còn một thao tác là bật/tắt hiragana" đúng khi người dùng ngồi sẵn ở layout Nhật. TrioKey không đổi layout hộ: ở layout ENG nó gõ Việt/Anh, muốn sang Nhật người dùng vẫn tự chuyển layout như hiện nay, chỉ là không còn phải tắt V.

### 5.3 Nhận biết trạng thái IME

- Luật cho tiếng Nhật: nhường khi và chỉ khi ngôn ngữ layout của luồng đang focus là Nhật **và** open status = 1.
- Luật cho Trung/Hàn (có unit test, chưa gõ thử tay; xem AT2-CK trong `docs/HANDS_ON_TEST.md`): nhường khi open status = 1 và conversion mode có cờ NATIVE.
- Đường xử lý phím chỉ đọc một giá trị đã lưu. Một luồng riêng làm mới giá trị đó khi: focus hoặc cửa sổ trước đổi; sau bất kỳ phím hay cú nhấp chuột nào có thể đã đổi trạng thái IME; và theo nhịp ngắn trong lúc layout đang là CJK. Không cần biết người dùng gán phím nào để bật/tắt IME: mọi thứ không phải phím ký tự thường đều được coi là "có thể đã đổi".
- Hook thấy phím bật/tắt trước khi ứng dụng kịp xử lý nó, nên đọc ngay lúc đó sẽ ra trạng thái cũ. Phải đọc lại sau khi ứng dụng đã xử lý phím. Ngoại lệ duy nhất cho quy tắc "không hỏi trên đường xử lý phím": phím ký tự đầu tiên sau một phím "có thể đã đổi" được phép chờ kết quả đọc mới, tối đa 30 ms; quá hạn thì dùng giá trị đã biết gần nhất. Khi hai yêu cầu va nhau, đúng trạng thái (AT3) thắng độ trễ (AT7).
- Mọi luật ở mục này chỉ áp dụng khi layout là CJK; layout khác thì không cần đọc gì và luôn gõ Việt (5.2). Với một cửa sổ ở layout CJK: lần đọc bị quá giờ thì giữ giá trị đã biết gần nhất của cửa sổ đó. Nếu chưa từng đọc được giá trị nào cho cửa sổ đó (bị từ chối, không có cửa sổ IME, hoặc quá giờ ngay từ lần đầu) thì coi như IME đang mở và nhường, tức là quay về đúng hành vi của VKey hiện nay. Người dùng vẫn gõ Việt được ở layout ENG.
- Khi trạng thái đổi giữa chừng một từ: bỏ từ đang gõ dở, không sửa gì thêm.

### 5.4 Lõi gõ

**Mô hình.** Lõi giữ dãy phím gốc của từ đang gõ. Sau mỗi phím nó tính lại từ đầu "dãy phím này nên hiện thành gì", so với chữ đang có trên màn hình, rồi trả về một lệnh sửa: xoá N ký tự cuối, chèn chuỗi S. Không có trạng thái tăng dần nào khác, nên không có lớp lỗi "trạng thái lệch với màn hình". Chữ xuất ra là Unicode dựng sẵn (mỗi chữ cái có dấu là một ký tự, một lần xoá lùi).

```rust
pub struct Edit { pub delete: usize, pub insert: String }

impl Engine {
    pub fn key(&mut self, ch: char) -> Edit;       // một chữ cái
    pub fn boundary(&mut self, ch: char) -> Edit;  // dấu cách, dấu câu, Enter…: chốt từ
    pub fn backspace(&mut self) -> Edit;           // xoá lùi, hoặc hoàn tác (xem dưới)
    pub fn reset(&mut self);                       // con trỏ đã ở chỗ khác
}
```

**Telex.** Chuẩn tham chiếu khi mơ hồ là hành vi Telex của UniKey.

| Phím | Kết quả |
| --- | --- |
| `aa` `aw` `ee` `oo` `ow` `uw` `dd` | â ă ê ô ơ ư đ |
| `w` đứng riêng | ư |
| `uow` | ươ |
| `s` `f` `r` `x` `j` | sắc, huyền, hỏi, ngã, nặng |
| `z` | xoá dấu thanh |
| Gõ lặp một phím dấu | huỷ dấu đó (xem "Phím dấu gõ lặp" bên dưới) |

- Phím dấu gõ ngay sau nguyên âm hoặc ở cuối từ đều được (`tieesng`, `tieengs` → `tiếng`).
- Vị trí dấu thanh với `oa`, `oe`, `uy`: mặc định kiểu cũ (`hòa`, `thúy`) như người dùng đang đặt; có lựa chọn kiểu mới.
- `[` và `]` không bị ánh xạ.
- Giữ nguyên kiểu chữ hoa/thường của người gõ.

**Trong lúc gõ.** Biến đổi được áp dụng ngay chừng nào kết quả còn là phần đầu của một âm tiết tiếng Việt hợp lệ. Khi dãy phím không thể là tiếng Việt nữa, hiện lại *dạng phím gốc* của từ và giữ như vậy tới hết từ. Dạng phím gốc là đúng các phím đã gõ, với một ngoại lệ nêu ở "Phím dấu gõ lặp".

**Phím dấu gõ lặp.** Theo cơ chế đã đo của Vikey (D17–D19). Gõ lặp một phím dấu huỷ dấu đó:

- nếu sau khi huỷ kết quả vẫn là một âm tiết tiếng Việt thì nó là tiếng Việt (`xooong` → `xoong`);
- nếu không, trong lúc gõ màn hình hiện kết quả Telex kinh điển, tức phím lặp chỉ hiện một lần (`verr` → `ver`), trừ khi dãy phím với cả hai lần bấm đã là một từ tiếng Anh (`err`, `boss`, `less`) thì hiện đủ mọi phím;
- khi chốt từ, danh sách từ tiếng Anh quyết định theo cả hai chiều: chỉ dạng hai-lần-bấm là từ tiếng Anh thì trả đủ mọi phím (`terrible`, `sorry`, `tomorrow`); chỉ dạng một-lần-bấm là từ tiếng Anh thì bỏ phím lặp (`mixx` → `mix`); cả hai hoặc không dạng nào là từ tiếng Anh thì giữ như đang hiện.

Cái giá đã biết: một tên riêng có chữ lặp nằm ngoài danh sách sẽ mất một chữ (`Larry` → `Lary`, `Nassr` → `Nasr`). Đổi lại, lỗi gõ tiếng Việt có phím dấu lặp cho ra kết quả Telex quen tay (`bijj` → `bij`). File "luôn là tiếng Anh" của người dùng gỡ được từng trường hợp.

**Khi chốt từ**, quyết định theo thứ tự:

1. *Mã* chứ không phải văn xuôi thì ở dạng phím gốc. Luật này chạy từ trái sang phải trong một cụm liền nhau (giữa hai khoảng trắng) và không sửa lại phần đã chốt:
   - Cụm bắt đầu bằng `-`, `/`, `\`, `.`, `~`, `$`, `@`, `#` là mã ngay từ đầu (`-las`, `--force`, `/usr/bin`).
   - Trong cụm xuất hiện chữ số hoặc `_`, hoặc từ đang gõ hoa-thường lẫn lộn (`has123`, `my_var`, `iPhone`): từ đang gõ và phần còn lại của cụm là mã.
   - Sau `.`, `:` hoặc `\` mà gõ tiếp chữ ngay, không cách (`obj.dir`, `C:\tmp`, `https://…`): phần còn lại của cụm là mã. Phần đứng trước đã chốt như một từ bình thường và giữ nguyên.
   - `/` và `-` nằm giữa hai chữ là ranh giới từ bình thường, mỗi bên được xét riêng (`bật/tắt`, `Anh/Việt`, `Việt-Nhật`). Nhưng khi một bên rõ ràng là tiếng Anh thì phần còn lại của cụm là mã (`src/main`, `docs/las`).
2. Kết quả Telex không nằm trong tập âm tiết tiếng Việt thì từ ở dạng phím gốc (`windows`, `text`, `less`).
3. Nằm trong tập âm tiết và dãy phím không phải từ tiếng Anh thì là tiếng Việt.
4. Cả hai đều đúng (`this`/`thí`, `has`/`há`, `been`/`bên`): theo từ đứng ngay trước nếu từ đó rõ ràng thuộc một thứ tiếng; nếu không thì theo mặc định ghi sẵn cho từng từ.

"Rõ ràng tiếng Việt" là từ có dấu; "rõ ràng tiếng Anh" là từ không phải âm tiết tiếng Việt; còn lại là trung tính và bị bỏ qua.

**Hoàn tác.** Lần Backspace đầu tiên ngay sau khi chốt từ sẽ đổi lại cách đọc, trong hai trường hợp: từ thuộc nhóm 4, hoặc từ vừa bị luật "Phím dấu gõ lặp" sửa lúc chốt. Backspace lần nữa mới xoá ký tự. Với mọi từ khác Backspace là xoá lùi bình thường.

**Xoá lùi giữa từ.** Xoá một ký tự trên màn hình, rồi suy lại dãy phím gốc từ phần chữ còn lại.

**Bỏ từ đang gõ dở** (không sửa gì nữa) khi: nhấp chuột, đổi focus, phím mũi tên / Home / End / Delete, tổ hợp có Ctrl / Alt / Win, Esc.

**Phím do phần mềm khác gửi vào** (remote desktop, KVM mềm, phần mềm đổi phím) được xử lý như phím thật, để gõ được tiếng Việt qua các công cụ đó. Riêng ký tự Unicode mà phần mềm khác chèn thẳng thì không phải phím Telex: chỉ làm bỏ từ đang gõ dở.

**Dữ liệu.** Lúc chạy mang hai bảng:

- Tập âm tiết tiếng Việt (khoảng 7 nghìn mục).
- Danh sách từ tiếng Anh thông dụng kèm tần suất (cỡ vài chục nghìn từ), dùng cho ba việc: nhận ra từ trùng ở luật 4, đặt mặc định Anh hay Việt cho từng từ trùng, và quyết định ở "Phím dấu gõ lặp". Mặc định của nhóm từ trùng hay gặp nhất được rà tay.

Nguồn dữ liệu phải có giấy phép cho dùng lại và được ghi rõ trong repo. Không chép code hay dữ liệu từ VKey và Vikey (GPL-3.0). Người dùng có hai file văn bản để ép: từ luôn là tiếng Việt, từ luôn là tiếng Anh.

### 5.5 Gửi chữ ra ứng dụng

**Mặc định.** Sửa ngay khi gõ: phím không làm đổi chữ thì cho đi thẳng, không chạm vào; phím làm đổi chữ thì bị chặn, thay bằng `N × Backspace` + chuỗi Unicode gửi bằng `SendInput` ngay trong callback của hook, để không phím nào gõ sau chen vào giữa.

**Ô có tự điền.** Thanh địa chỉ trình duyệt và ô Excel tự chèn phần gợi ý sau con trỏ, làm số lần xoá lùi bị lệch. Với các tiến trình trong một danh sách (file văn bản, có sẵn danh sách khởi đầu), TrioKey huỷ gợi ý trước khi xoá. Đạt khi qua AT5.

**Terminal.** Không có xử lý riêng (mục 2.2): terminal dùng cách gửi mặc định.

### 5.6 Ô không nhìn thấy chữ

Không còn chế độ V/E nên phải nói rõ chuyện gì xảy ra với mật khẩu:

- Ô mật khẩu Win32 (`ES_PASSWORD`): TrioKey tắt hẳn.
- Ô mật khẩu trong trình duyệt và Electron: TrioKey không nhận ra được. Một đoạn của mật khẩu chỉ bị đổi khi nó là chữ cái thuần, kiểu hoa/thường bình thường, đứng trước chữ số hay `_` đầu tiên, và gõ Telex ra đúng một âm tiết tiếng Việt (ví dụ đoạn `mas` trong `mas!x9` thành `má`). Phần còn lại không bị đụng tới nhờ luật 1 ở mục 5.4.
- Đường lui: phím tắt bật/tắt, và biểu tượng khay cho biết đang ở trạng thái nào.

### 5.7 Cài đặt và khay hệ thống

```
┌ TrioKey – Cài đặt ──────────────────────────────────────┐
│ [x] Bật gõ tiếng Việt          Phím tắt [Ctrl+Shift ▾] │
│ Kiểu bỏ dấu   (•) hòa, thúy    ( ) hoà, thuý           │
│ [x] Tự giữ nguyên từ tiếng Anh                         │
│ [x] Nhường khi IME Nhật/Trung/Hàn đang bật             │
│ [x] Khởi động cùng Windows                             │
│ [ ] Chạy quyền admin (gõ được trong cửa sổ admin)      │
│ [x] Tự động cập nhật               [Kiểm tra bản mới]  │
│ (dòng kết quả kiểm tra cập nhật)                       │
│ Mở thư mục cấu hình…                          v0.1.0   │
└────────────────────────────────────────────────────────┘
Khay: biểu tượng V / あ / –     Menu: Bật-tắt · Cài đặt… · [Có bản X.Y.Z — …] · Thoát
```

Phím tắt chọn từ vài tổ hợp có sẵn, không có ô ghi phím. Cửa sổ dựng bằng Win32 thuần, không dùng framework giao diện. Cấu hình là file văn bản trong `%APPDATA%\triokey`. Lần chạy đầu chép cấu hình từ thư mục tên cũ `%APPDATA%\evj` nếu có.

### 5.8 Hiệu năng, riêng tư, độ bền

- Callback của hook: p99 dưới 1 ms (không tính phím đầu tiên sau khi bật/tắt IME ở 5.3), không đọc/ghi file. Có số đo, không tối ưu vi mô.
- Phím do chính TrioKey gửi được đánh dấu và bỏ qua khi quay lại hook.
- Windows tự gỡ hook nào trả lời chậm; TrioKey phải tự phát hiện và cài lại.
- Không ghi phím (mục 2.3).
- Mạng: TrioKey chỉ kết nối tới GitHub, và chỉ để kiểm tra và tải bản cập nhật có chữ ký (G8). Chi tiết ở `docs/decisions/0007-auto-update.md`; các điểm bắt buộc:
  - Hai địa chỉ cố định dưới `https://github.com/ved0el/triokey/releases/latest/download/`: `triokey.exe.minisig` rồi `triokey.exe`, mỗi file tải một lần cho một lần kiểm tra (qua các lần chuyển hướng của GitHub). Chỉ HTTPS; mỗi lần chuyển hướng đều phải tới `github.com` hoặc máy chủ tải bản phát hành của GitHub; giới hạn 20 MB và có hạn thời gian.
  - Yêu cầu là một lệnh GET trơn với `User-Agent: TrioKey-updater`: không gửi nội dung gõ, không gửi mã nhận dạng máy hay người dùng, không gửi cả số phiên bản.
  - Chỉ cài khi chữ ký minisign (Ed25519) đúng với khoá công khai biên dịch sẵn trong exe (`crates/triokey/update-pubkey.txt`), dòng "trusted comment" đã ký ghi đúng `file=triokey.exe`, và `version=` lớn hơn bản đang chạy. Không hạ cấp, không cài lại cùng phiên bản. Số phiên bản chỉ được hiện cho người dùng sau khi chữ ký đã được kiểm; trước đó dòng chưa kiểm chỉ quyết định có tải file exe hay không. File khoá còn là bản giữ chỗ thì TrioKey không kết nối, không tải, không cài.
  - Mọi việc mạng và file của phần cập nhật chạy trên một luồng riêng, không bao giờ trên luồng hook hay luồng giao diện. Kiểm tra mỗi 24 giờ một lần, lần thử đầu khoảng một phút sau khi chạy; lần nào hỏng (mất mạng, không đọc được file chữ ký) thì một giờ sau thử lại. Thời điểm kiểm tra gần nhất ghi trong `config.txt`; bản quyền admin chỉ giữ nó trong bộ nhớ (không tự ghi gì vào `%APPDATA%`), nên có thể kiểm tra thêm một lần sau mỗi lần chạy.
  - Tuỳ chọn `Tự động cập nhật` (`auto_update`, mặc định bật). Tắt đi thì không còn dùng mạng, trừ khi người dùng bấm `Kiểm tra bản mới` (kiểm tra một lần và, ở bản thường, cài luôn).
  - Bản thường: tải file exe vào bộ nhớ, kiểm chữ ký, ghi file đã kiểm vào chính thư mục chứa exe, đổi tên exe đang chạy thành `triokey.old.exe` rồi đặt file mới vào; có hiệu lực từ lần chạy sau, lúc đó file cũ bị xoá. Không ghi được vào thư mục, hoặc exe đang chạy không mang tên `triokey.exe`, thì chỉ báo có bản mới kèm đường tới trang phát hành.
  - Chế độ admin (5.9): không bao giờ tự cài. Tải file exe vào bộ nhớ, kiểm chữ ký rồi mới báo `Có bản X.Y.Z — bấm để cập nhật`; khi người dùng bấm thì tải lại, kiểm chữ ký và thay file ngay trong `%ProgramFiles%\TrioKey`. Không bao giờ kiểm một file ở nơi chương trình quyền thường ghi được rồi đem dùng với quyền admin. Bản thường không đụng tới bản admin. Một TrioKey quyền admin chạy từ thư mục khác thì chỉ báo (cũng sau khi đã kiểm chữ ký trong bộ nhớ), không ghi gì. "Quyền admin" ở đây là nửa nâng quyền của một token UAC bị tách; khi UAC tắt hẳn hoặc dùng tài khoản Administrator dựng sẵn thì không có phía quyền thường nào, và TrioKey xử sự như bản thường.

### 5.9 Quyền admin

Windows không cho một tiến trình quyền thường bắt hay gửi phím vào cửa sổ quyền admin. Tuỳ chọn "Chạy quyền admin" (mặc định tắt) làm TrioKey tự khởi động bằng một tác vụ lúc đăng nhập với quyền cao nhất; bật nó cần một lần đồng ý UAC.

Bắt buộc: file exe mà tác vụ đó chạy phải nằm ở nơi chỉ admin ghi được (ví dụ dưới `%ProgramFiles%`), nếu không bất kỳ chương trình nào cũng có thể đánh tráo exe để lấy quyền admin. Khi chạy quyền admin, TrioKey không mở file hay chương trình nào theo yêu cầu từ file cấu hình.

## 6. Nghiệm thu

Điều kiện chung cho mọi phép thử: thoát VKey và mọi bộ gõ tiếng Việt khác trước. TrioKey xử lý cả phím do phần mềm khác gửi vào (5.4), nên hai bộ gõ chạy cùng lúc sẽ sửa chữ của nhau.

| # | Phép thử | Đạt khi |
| --- | --- | --- |
| AT1 | Layout `0411`, IME đóng, gõ `tieesng vieejt` trong Notepad và Brave | ra `tiếng việt` |
| AT2 | Bật hiragana, gõ `nihongo`; tắt, gõ `tieesng`; không chạm phím tắt của TrioKey | ra `にほんご` rồi `tiếng`. Bắt buộc đạt trong Notepad và Brave. Một terminal, một ứng dụng Office và một ứng dụng Electron được thử và ghi lại kết quả; ứng dụng nào không cho đọc trạng thái IME thì rơi về 5.3, không chặn mốc |
| AT3 | Bấm phím bật/tắt hiragana rồi gõ ngay phím đầu tiên | 20/20 lần phím đầu được xử lý theo trạng thái mới |
| AT4 | (a) Bảng ca kiểm thử cho từng luật ở 5.4. (b) Gõ đoạn văn trộn Anh/Việt chuẩn | (a) đúng tuyệt đối. (b) không quá 3 lần sửa tay trên 100 từ; mỗi lần Backspace-hoàn-tác hoặc bấm phím tắt tính là một lần |
| AT5 | Gõ tiếng Việt trong thanh địa chỉ Brave và trong ô Excel | Bắt buộc đạt ở thanh địa chỉ Brave: đúng từng ký tự 20/20 lần. Ô Excel được thử và ghi lại kết quả, không chặn mốc |
| AT6 | So `HKCU\Keyboard Layout\Preload` và danh sách ngôn ngữ trước và sau khi chạy, bật tự khởi động, gỡ | không đổi |
| AT7 | Đo thời gian callback của hook khi gõ liên tục | p99 dưới 1 ms, với ngoại lệ nêu ở 5.8 |
| AT8 | Gõ `has` vào một ô mật khẩu Win32 | nhận đúng `has` |
| AT9 | Bật "Chạy quyền admin" rồi gõ tiếng Việt trong một cửa sổ admin | gõ được; file exe của tác vụ nằm ở nơi chỉ admin ghi được |
| AT10 | Với một bản phát hành thử mới hơn bản đang chạy: để TrioKey tự kiểm tra; lặp lại ở chế độ admin, với `Tự động cập nhật` tắt, và với bản build dùng khoá giữ chỗ | bản thường tự thay exe và báo có hiệu lực lần chạy sau; bản admin chỉ cài sau khi bấm; khi tắt hoặc khi không có khoá thì không có kết nối mạng nào |

### 6.1 Khi nào dừng, khi nào chấp nhận xuống cấp

**Dừng và quay lại hỏi người dùng:**

- AT1 trượt trong Notepad hoặc Brave. Khi đó "direct input ở layout Nhật thì gõ Việt" không làm được bằng cách chèn chữ; phương án còn lại là chỉ gõ Việt ở layout không phải CJK.
- AT3 không đạt dù đã dùng ngoại lệ 30 ms ở 5.3.
- Sau hai vòng tinh chỉnh ở M3 mà AT4(b) vẫn trên 3 lần sửa tay / 100 từ. Khi đó bàn lại hướng C.

**Chấp nhận, không chặn mốc:**

- Một ứng dụng không cho đọc trạng thái IME: hành xử như VKey (5.3).
- Một ứng dụng xử lý xoá-chèn kém mà chưa có cách gửi đạt tiêu chí: ghi vào danh sách đã biết, người dùng tắt tay ở ứng dụng đó.

## 7. Thứ tự làm

| Mốc | Nội dung | Người dùng thử được gì |
| --- | --- | --- |
| M1 | `engine`: kiểm tra tương thích của `vi` rồi chốt lõi bỏ dấu; tập âm tiết; trả lại phím khi không phải tiếng Việt; test | chưa có gì để gõ |
| M2 | `triokey.exe`: hook, gửi chữ (kể cả ô có tự điền), khay, phím tắt, nhường IME, ô mật khẩu Win32 | thay được VKey cho việc gõ Việt + Nhật; từ trùng còn nghiêng về tiếng Việt cho tới M3 (AT1–AT3, AT5–AT8) |
| M3 | Tự chuyển Anh/Việt: luật cụm, từ trùng, ngữ cảnh từ trước, hoàn tác | gõ trộn Anh/Việt không bấm chuyển (AT4) |
| M4 | Cửa sổ cài đặt, tự khởi động, chế độ admin | bản dùng hằng ngày (AT9) |
| M5 | Tự cập nhật có chữ ký, CI và quy trình phát hành trên GitHub | bản tự cập nhật (AT10); cần khoá ký và một bản phát hành trước |

Theo yêu cầu người dùng, công việc chỉ dừng khi cần gõ thử trực tiếp. Mọi phần kiểm được bằng `cargo test` (M1 và phần lõi của M3) làm trước; lần chạy `triokey.exe` đầu tiên là điểm dừng đầu tiên, và mọi thao tác cần UAC cũng là điểm dừng. Agent không bao giờ chạy hook hay `SendInput` trên máy của người dùng: chỉ biên dịch và `cargo test`.

## 8. Để lại cho người làm

- Bố cục crate, kiểu dữ liệu bên trong, định dạng file cấu hình và file dữ liệu.
- Bảng âm đầu / vần / thanh hợp lệ, và nguồn cụ thể của tập âm tiết và danh sách tần suất (phải thoả điều kiện giấy phép ở 5.4).
- Lớp ký tự chính xác cho "ranh giới từ" và "dấu câu ở rìa", kèm ca kiểm thử.
- Nhịp làm mới trạng thái IME; cách huỷ gợi ý và danh sách tiến trình khởi đầu dùng cách đó: lấy từ đo đạc.
- Cách tự kiểm tra `triokey.exe` (ví dụ một chế độ tự gõ vào cửa sổ thử của chính nó rồi so kết quả).
- Toolchain: đã cài theo lời người dùng ngày 2026-11-25 bằng `mise use rust@stable-x86_64-pc-windows-gnu` trong thư mục dự án (rustc 1.100.0, host GNU, không cần Visual Studio Build Tools; cấu hình ở `mise.toml`). Đã build và chạy thử một chương trình dùng `vi` 0.8.0 và `windows-sys`. Mọi thứ khác cần cài thêm lên máy thì hỏi người dùng trước.

## 9. Rủi ro còn mở

1. Trạng thái IME không đọc được ở một số ứng dụng. Khi đó ứng dụng ấy hành xử như VKey hiện nay (5.3).
2. Chất lượng tự chuyển Anh/Việt ở bản đầu sẽ kém Vikey; bù bằng hoàn tác một phím và hai file ép từ. Đường lui là hướng C.
3. Terminal không có xử lý riêng: gõ tiếng Việt trong herdr/ssh có thể vẫn lỗi như với VKey. Người dùng đã chấp nhận.
4. File exe bắt phím, không ký số, có thể bị phần mềm diệt virus cảnh báo.
5. Thư viện `vi` có thể không qua bài kiểm tra tương thích đầy đủ; khi đó tự viết lõi bỏ dấu và M1 dài hơn.

## 10. Lựa chọn của người viết, chờ người dùng soát

Những điểm dưới đây người dùng chưa trực tiếp quyết; sửa điểm nào thì báo, phần còn lại của thiết kế không đổi.

1. Người dùng bảo bỏ qua herdr và ssh; người viết giữ lại xử lý ô có tự điền. Chế độ admin đã được người dùng yêu cầu lại, không còn là lựa chọn của người viết.
2. Tên: đã chốt là TrioKey (tên tạm cũ: EVJ).
3. Hướng A: keyboard hook + lõi Rust tự viết, không dùng code GPL.
4. Chỉ Telex; `w` đứng riêng ra `ư`; `[` `]` không ánh xạ; bỏ dấu kiểu cũ là mặc định.
5. Phím dấu gõ lặp: theo cơ chế của Vikey như người dùng yêu cầu. Điểm này đảo ví dụ đã đưa người dùng xem trước đó: `Larry` nay ra `Lary` nếu tên không có trong danh sách từ tiếng Anh.
6. `/` và `-` giữa hai chữ là ranh giới từ (`bật/tắt`); cụm bắt đầu bằng `-` hay `/` là mã (`-las`). Luật mã chạy từ trái sang phải, không sửa lại phần đã chốt.
7. Hoàn tác bằng Backspace, áp dụng cho từ có hai cách đọc và cho từ vừa bị luật phím lặp sửa lúc chốt.
8. Ở layout CJK mà không đọc được trạng thái IME thì nhường, như VKey.
9. Phím do phần mềm khác gửi vào vẫn được xử lý như phím thật.
10. Phím ký tự đầu tiên sau khi bật/tắt IME được chờ tối đa 30 ms.
11. Ngưỡng đạt của tự chuyển Anh/Việt: không quá 3 lần sửa tay trên 100 từ.
12. Ô có tự điền: thanh địa chỉ Brave là bắt buộc, Excel chỉ ghi nhận kết quả.
13. Phím tắt chọn từ danh sách có sẵn; cấu hình nâng cao bằng file văn bản.
14. Toolchain Rust bản GNU, để khỏi cài Visual Studio Build Tools.
