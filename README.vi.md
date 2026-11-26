<p align="center"><img src="https://raw.githubusercontent.com/ved0el/triokey/main/assets/brand/social-preview.png" width="640" alt="TrioKey: bộ gõ tiếng Việt cho Windows"></p>

<p align="center">
  <a href="https://github.com/ved0el/triokey/actions/workflows/ci.yml"><img src="https://github.com/ved0el/triokey/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/ved0el/triokey/releases/latest"><img src="https://img.shields.io/github/v/release/ved0el/triokey" alt="Bản phát hành mới nhất"></a>
  <a href="https://github.com/ved0el/triokey/releases"><img src="https://img.shields.io/github/downloads/ved0el/triokey/total" alt="Lượt tải"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/ved0el/triokey" alt="Giấy phép: MIT"></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-3565B8" alt="Nền tảng: Windows 10 và 11">
</p>

# TrioKey

[English](README.md) · **Tiếng Việt**

Bộ gõ tiếng Việt cho Windows, dành cho người ngày nào cũng gõ cả tiếng Anh, tiếng Việt và một trong ba thứ tiếng Trung, Nhật, Hàn (CJK). Khi gõ trộn hằng ngày, bạn không phải bật/tắt bộ gõ tiếng Việt: nút duy nhất cần bấm là nút bật/tắt của chính IME CJK.

```text
phím gõ             chữ hiện ra
tieesng vieejt␣     tiếng việt
windows␣            windows
songs␣              sóng         (đầu dòng mới: cách đọc phổ biến hơn thắng)
the songs␣          the songs    (sau một từ tiếng Anh)
this␣⌫              thí          (một lần Backspace đổi từ mà bộ gõ còn phân vân)
```

Bản đầu: 0.2.1 đã phát hành và có chữ ký, các ví dụ trên trang này mô tả lõi gõ của bản đó. Nhiều phần của ứng dụng Windows, kể cả phần tự cập nhật đang bật sẵn, chưa được kiểm trên máy thật. Xin đọc mục [Tình trạng](#tình-trạng) trước.

## Vì sao có TrioKey

**1. Một ý định, hai lần bật/tắt.**
Với các bộ gõ khác, muốn chen một từ tiếng Nhật giữa câu tiếng Việt thì phải bật hiragana, rồi còn phải tắt tiếng Việt, nếu không Telex sẽ làm hỏng romaji; lúc quay lại thì làm ngược cả hai.
Với TrioKey: bật hiragana, gõ, tắt hiragana. TrioKey đọc xem IME CJK có thực sự đang mở trong cửa sổ bạn đang gõ hay không rồi tự nhường. Khi IME đóng (direct input), nó gõ tiếng Việt, kể cả khi bàn phím đang ở layout Nhật.

**2. Không có chế độ Anh/Việt.**
Mỗi từ được quyết định khi bạn gõ xong từ đó. TrioKey so dãy phím gõ nguyên với cách đọc Telex của nó, dựa trên tần suất mỗi bên được dùng như một từ trong văn bản tiếng Anh và tiếng Việt, và trên việc dòng đang gõ nghiêng về tiếng Việt tới đâu. Khi còn phân vân, một lần Backspace ngay sau từ sẽ đổi sang cách đọc kia, và lần đổi nào bạn giữ lại thì được nhớ.

**3. Không đụng vào thiết lập bàn phím của Windows.**
TrioKey không thêm layout bàn phím, không đăng ký text service (TSF), không đụng vào danh sách bàn phím của Windows. Nó chỉ là một ứng dụng nhỏ ở khay hệ thống: một keyboard hook (kèm một hook chuột chỉ để biết có cú nhấp) và phần gửi chữ.

**4. Cửa sổ chạy quyền admin.**
Bộ gõ chạy quyền thường không gõ được vào cửa sổ quyền admin. TrioKey có chế độ admin tuỳ chọn: một lần đồng ý UAC, một tác vụ chạy lúc đăng nhập, và file exe được chép vào thư mục chỉ admin ghi được. (Chưa từng chạy trên máy thật.)

**5. Cố ý giữ nhỏ.**
Không gõ tắt, không nhớ chế độ theo ứng dụng, không cloud, không thu thập dữ liệu, không nhật ký gõ. Chỗ duy nhất dùng mạng là việc kiểm tra bản cập nhật có chữ ký trên GitHub, và bạn tắt được. Một cửa sổ cài đặt nhỏ và vài file văn bản thường.

## Tình trạng

- **Đã phát hành:** [0.2.1](https://github.com/ved0el/triokey/releases), do CI build và ký ([CHANGELOG.md](CHANGELOG.md)). Bản 0.2.0 đưa vào lõi gõ mô tả trên trang này (tần suất từ và ngôn ngữ của dòng), việc nhớ các lần đổi đã giữ, biểu tượng khay đỏ/xanh dương và biểu tượng ứng dụng mới; bản 0.2.1 làm lại cửa sổ cài đặt. 0.1.0 và 0.1.1 dùng lõi gõ cũ, quyết định từ theo một danh sách từ và từ đứng trước.
- **Tác giả đã gõ thử:** bản 0.2.0 đã được dùng để gõ văn bản trộn Anh/Việt thông thường trên một máy Windows 11 và được coi là đủ tốt để phát hành; vẫn còn vài từ ra sai. Trong lần gõ đó không có lần đổi nào được nhớ, và không dòng nào của bảng kiểm được chạy, trừ một lần đọc thời gian (AT7).
- **Đã gõ thử bằng tay:** một bản dựng trước đây với tiếng Nhật (Microsoft IME, trong Notepad và Brave) đạt AT1 và AT2 của [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md). Cả hai chưa được làm lại với 0.2.0.
- **Chưa được kiểm trên máy thật** (chưa dòng nào của bảng kiểm có kết quả ghi lại), mới chỉ biên dịch và unit test ở chỗ làm được: biểu tượng khay và menu, biểu tượng ứng dụng, các phím tắt, cửa sổ cài đặt (giao diện sáng và tối mới chỉ được xem qua ảnh do một test vẽ ra ở 96 DPI, chưa từng trên màn hình thật; giao diện tương phản cao chưa từng được xem khi bật một chủ đề tương phản), khởi động cùng Windows, chế độ admin, phần tự cập nhật (AT10 trong [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) vẫn chưa làm), việc lưu các từ đã nhớ, các bước gỡ bỏ và việc bỏ qua ô mật khẩu Win32. Các mục bên dưới mô tả chúng theo mã nguồn. Nếu không muốn một phần tự cập nhật chưa được thử thay file exe của bạn, hãy bỏ chọn `Tự động cập nhật` trong Cài đặt.
- **Tiếng Trung và tiếng Hàn:** mới chỉ có unit test cho luật quyết định (IME mở và đang ở chế độ bản ngữ), với giá trị giả. Các IME đó có báo đúng trạng thái hay không thì chưa ai thử.
- Mới thử trên Windows 11; Windows 10 chưa thử. Chỉ có Telex, chưa có VNI. Giao diện (menu khay, tooltip, cửa sổ cài đặt, thông báo) chỉ có tiếng Việt.
- **Số đo**, có được bằng cách cho chữ chạy qua lõi gõ trong một bài test (không phải trên máy thật): trên 10.274 từ văn bản trộn Anh/Việt do những người chưa từng thấy lõi gõ viết ra, tỉ lệ từ sai giảm từ 0,54% với lõi gõ của 0.1.1 xuống 0,16% với lõi hiện tại, và số phím phải bấm để sửa từ giảm từ 2,74 xuống 0,59 trên 100 từ. Khi bật việc nhớ, số từ sai giảm từ 16 xuống 12 và không từ nào phải sửa tới lần thứ hai. Bộ test hồi quy của VieLotus-IM, gồm những từ đơn khó gõ không có ngữ cảnh và chỉ dùng để canh không cho lõi gõ tụt lùi, tăng từ 8.361 lên 8.524 trên 9.084. Cách chạy cả hai: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), mục 11.
- Terminal không có xử lý riêng. Các ô tự chèn gợi ý sau con trỏ (thanh địa chỉ trình duyệt, ô Excel) cũng vậy.
- Không nhận ra được ô mật khẩu trong trình duyệt và ứng dụng Electron; xem [Riêng tư và bảo mật](#riêng-tư-và-bảo-mật).
- Một số ứng dụng giấu trạng thái IME. Khi không đọc được, TrioKey dùng trạng thái đọc được gần nhất của cửa sổ đó; nếu chưa có thì trên layout CJK nó nhường.
- macOS và Linux chưa bắt đầu.

## TrioKey gõ ra gì trong từng trường hợp

| Layout bàn phím của cửa sổ | Trạng thái IME CJK | Phím bạn gõ sẽ ra gì | Khay |
| --- | --- | --- | --- |
| Tiếng Anh, hoặc layout không phải CJK | (không liên quan) | Telex tiếng Việt, từ tiếng Anh giữ nguyên như đã gõ | `V` |
| Nhật | đóng (direct input) | Telex tiếng Việt, từ tiếng Anh giữ nguyên như đã gõ | `V` |
| Nhật | mở (hiragana, katakana…) | mọi phím đi thẳng tới IME, không bị đụng tới | `E` |
| Trung hoặc Hàn † | đóng, hoặc mở ở chế độ chữ Latin | Telex tiếng Việt, từ tiếng Anh giữ nguyên như đã gõ | `V` |
| Trung hoặc Hàn † | mở ở chế độ bản ngữ | mọi phím đi thẳng tới IME, không bị đụng tới | `E` |
| CJK, trong ứng dụng giấu trạng thái IME | không biết | TrioKey nhường | `E` |
| bất kỳ, sau khi bạn tắt TrioKey | (không liên quan) | mọi phím đi thẳng | `E` |

† Chưa thử với IME thật: đây là luật trong mã nguồn, mới chỉ được kiểm bằng unit test. Các dòng tiếng Nhật đã được gõ thử trên một bản dựng trước đây (xem [Tình trạng](#tình-trạng)).

TrioKey không đổi layout bàn phím hộ bạn. "Chỉ còn một nút" đúng khi bạn ngồi sẵn ở layout CJK và dùng nút bật/tắt của IME; nếu bạn đang ở layout tiếng Anh thì vẫn phải tự chuyển layout để sang tiếng Nhật, chỉ là không còn phải tắt tiếng Việt nữa.

### Tiếng Anh hay tiếng Việt

Khi một từ kết thúc (dấu cách, Enter, dấu câu), TrioKey có hai cách đọc: dãy phím gõ nguyên, và kết quả Telex của nó. Mỗi cách được chấm theo tần suất được dùng như một từ, lấy từ các bảng tần suất dựng từ văn bản trên web của mỗi thứ tiếng, cộng một danh sách từ điển tiếng Anh và một danh sách thuật ngữ kỹ thuật. Dòng đang gõ có một độ nghiêng: mỗi từ gõ xong đẩy nó về phía tiếng Việt hoặc tiếng Anh, còn Enter hay việc dời con trỏ thì mở một dòng mới, hơi nghiêng về tiếng Việt. Cách đọc nào có khả năng cao hơn trên dòng này thì thắng. Từ có chữ số, `_` hoặc chữ hoa lẫn giữa từ, đường dẫn, tham số dòng lệnh và chữ ngay sau `.` `:` `\` được coi là mã và giữ nguyên.

Một lần Backspace ngay sau từ sẽ đổi từ đó sang cách đọc kia và xoá luôn dấu cách, nhưng chỉ khi bộ gõ còn phân vân: cách đọc kia không kém xa về khả năng, hoặc đó là một từ tiếng Anh thông dụng vừa thua tiếng Việt. Từ trong các file ép từ, từ đã nhớ và từ VIẾT HOA được giữ nguyên thì luôn cho đổi, nếu từ đó có cách đọc thứ hai. Sau một từ mà bộ gõ đã chắc, Backspace chỉ xoá dấu cách. Backspace lần thứ hai bao giờ cũng xoá như thường.

### Ví dụ gõ

Mọi dòng dưới đây đều được test `readme_examples` trong `crates/triokey-engine/tests/engine.rs` kiểm. Mỗi dòng được gõ ở đầu một dòng mới. `␣` là phím cách, `⌫` là Backspace; dấu cách mà `␣` cuối cùng để lại không được ghi ra.

```text
phím gõ                        chữ hiện ra
tieesng vieejt␣                tiếng việt
windows terrible␣              windows terrible
DDuowngf␣                      Đường
baajt/taawst␣                  bật/tắt
my_var obj.dir src/main␣       my_var obj.dir src/main       (mã được để nguyên)
server cos␣                    server có          (có phổ biến hơn hẳn, kể cả sau tiếng Anh)

songs␣                         sóng               (đúng cả hai cách: dòng mới nghiêng về tiếng Việt)
the songs␣                     the songs          (sau một từ tiếng Anh)
tooi thaasy songs␣             tôi thấy sóng      (sau các từ tiếng Việt)

this␣                          this
this␣⌫                         thí                (còn phân vân: một lần Backspace đổi)
this␣⌫⌫                        th                 (Backspace lần hai thì xoá)
off␣⌫                          of
tooi␣⌫                         tôi                (đã chắc: Backspace chỉ xoá dấu cách)
tooi thaasy sets␣              tôi thấy sét       (từ tiếng Anh thông dụng sau các từ tiếng Việt...)
tooi thaasy sets␣⌫             tôi thấy sets      (...luôn chỉ cách một lần Backspace)
There␣                         Thể                (từ đầu dòng có thể bị đoán sai...)
There␣⌫                        There              (...một lần Backspace là trả lại)

mixx␣                          mix                (phím dấu gõ lặp thì huỷ dấu)
offline passwd Terraform␣      offline passwd Terraform
ko ddc ddaau␣                  ko đc đâu          (chữ viết tắt thông dụng có đ thì được ghép)
dd/mm/yyyy aa aw␣              dd/mm/yyyy aa aw   (dd, aa, aw đứng riêng thì giữ nguyên)
toansz␣                        toan               (z xoá dấu thanh)
OS naof cungx dduwowcj␣        OS nào cũng được   (VIẾT HOA giữ nguyên, trừ từ tiếng Việt thông dụng)
VIEEJT␣                        VIỆT
chaau As␣                      châu Á             (từ một chữ cái: chỉ giữ Shift cho chữ cái)
Mac OS␣                        Mac OS             (hai chữ hoa là từ viết tắt, ở đâu cũng vậy)
```

## Khác gì các bộ gõ khác

Tính đến tháng 11 năm 2026. VKey, Vikey và VieLotus-IM được mô tả theo việc đọc repo và tài liệu của chúng; không bộ nào được cài hay chạy thử. UniKey, EVKey và OpenKey không được đọc; chúng chỉ được nhắc tới vì chế độ Việt/Anh có phím chuyển mà ai cũng biết.

| | TrioKey | Cách làm thường gặp ở nơi khác |
| --- | --- | --- |
| Nhường IME CJK | đọc trạng thái mở/đóng của IME trong cửa sổ đang gõ | nhường bất cứ khi nào layout CJK đang bật (VKey, Vikey), hoặc là một mục riêng trong danh sách bàn phím của Windows (VieLotus-IM) |
| Từ tiếng Anh trong câu tiếng Việt | không có chế độ: xét từng từ theo tần suất từ và theo dòng | chế độ Việt/Anh có phím chuyển (UniKey, EVKey, OpenKey, VKey), hoặc tự nhận biết (Vikey, VieLotus-IM) |
| Cách gắn vào Windows | keyboard hook và gửi chữ, không đăng ký gì | như vậy ở các bộ gõ dùng hook; text service TSF ở VieLotus-IM và ở thành phần tuỳ chọn của VKey |
| Kiểu gõ, gõ tắt | chỉ Telex, không gõ tắt | VNI và các kiểu gõ khác khá phổ biến; VKey và VieLotus-IM có gõ tắt |

Các bộ gõ kia đi xa hơn: có gói cài, đã được phát hành tới người dùng, có nhiều kiểu gõ hơn và một số chạy trên nhiều nền tảng hơn. TrioKey học được từ vài bộ trong số đó; xem [Ghi nhận](#ghi-nhận).

## Cài đặt

Chưa có bộ cài. Các bản phát hành do CI build và ký từ một tag phiên bản ([docs/RELEASING.md](docs/RELEASING.md)).

1. **Thoát mọi bộ gõ tiếng Việt khác trước** (UniKey, EVKey, OpenKey, VKey, Vikey…). Hai bộ gõ chạy cùng lúc sẽ sửa chữ của nhau.
2. Tải `triokey-X.Y.Z-windows-x64.zip` từ [bản phát hành mới nhất](https://github.com/ved0el/triokey/releases/latest) rồi giải nén vào một thư mục riêng, hoặc tự build từ mã nguồn (bên dưới).
3. Chạy `triokey.exe`. Không có cửa sổ nào hiện ra; ở khay hệ thống xuất hiện chữ `V` màu đỏ (có thể nằm sau mũi tên `^`). 0.1.0 và 0.1.1 vẽ chữ này một màu.

### Build từ mã nguồn

Chỉ hỗ trợ toolchain GNU: script build ghi tài nguyên của file exe (biểu tượng và manifest) chỉ cho `x86_64-pc-windows-gnu`, nên toolchain MSVC mặc định không build được TrioKey.

1. Cài Rust. Chọn một trong hai:
   - [mise](https://mise.jdx.dev): chạy `mise install` trong thư mục repo (nó đọc `mise.toml`, file này chọn toolchain GNU bản stable; không cần Visual Studio Build Tools), hoặc
   - [rustup](https://rustup.rs) với toolchain `stable-x86_64-pc-windows-gnu`.
2. Build:

   ```sh
   git clone https://github.com/ved0el/triokey
   cd triokey
   cargo build --release -p triokey        # dùng mise: mise x -- cargo build --release -p triokey
   ```

3. File exe nằm ở `target\release\triokey.exe`. Nó mang cùng khoá cập nhật với bản phát hành, nên cũng tự kiểm tra bản mới.

Cách chạy test và các bước kiểm mà CI chạy có trong [CONTRIBUTING.md](CONTRIBUTING.md).

File exe chưa được ký số. SmartScreen hoặc phần mềm diệt virus có thể cảnh báo, như với nhiều chương trình dùng keyboard hook khác. File `triokey.exe.minisig` đăng kèm mỗi file exe đã phát hành có thể kiểm bằng tay với [minisign](https://jedisct1.github.io/minisign/) và khoá công khai trong `crates/triokey/update-pubkey.txt`; phần tự cập nhật có sẵn cũng kiểm đúng như thế.

## Cách dùng

Mục này mô tả bản 0.2.1 (ở bản 0.2.0 cửa sổ cài đặt trông khác); biểu tượng đỏ/xanh dương, các luật Backspace hiện tại và việc nhớ từ chưa có trong 0.1.0 và 0.1.1. Phần lớn chưa được kiểm trên máy thật (xem [Tình trạng](#tình-trạng)).

- **Biểu tượng khay.** Chữ `V` màu đỏ là TrioKey đang gõ tiếng Việt. Chữ `E` màu xanh dương là phím đi thẳng, vì bạn đã tắt hoặc vì nó đang nhường IME CJK; di chuột lên để xem lý do (`Tiếng Anh — đã tắt` hoặc `Tiếng Anh — đang nhường IME Nhật`; với IME Trung và Hàn là `IME Trung`, `IME Hàn`). Chữ nằm trên nền trong suốt, màu đậm khi thanh tác vụ sáng và màu nhạt khi thanh tác vụ tối, và được vẽ lại khi bạn đổi chủ đề sáng/tối của Windows.
- **Nhấp trái** vào biểu tượng để bật/tắt TrioKey. **Nhấp phải** để mở menu: `Tiếng Việt` (bật/tắt), `Cài đặt…`, `Thoát`.
- **Phím tắt** bật/tắt: Ctrl+Shift (mặc định; bấm rồi nhả, không kèm phím nào khác), Alt+Z, Ctrl+Space, hoặc `Không dùng`. Bật/tắt bằng phím tắt hay ở khay không được lưu lại; chỉ ô `Bật gõ tiếng Việt` trong Cài đặt mới được lưu.
- **Cài đặt** (`Cài đặt…`): `Bật gõ tiếng Việt`, `Phím tắt`, `Kiểu bỏ dấu` (`hòa, thúy` hoặc `hoà, thuý`), `Tự giữ nguyên từ tiếng Anh` (tắt đi thì mọi từ đều gõ Telex thuần), `Nhường khi IME Nhật/Trung/Hàn đang bật`, `Khởi động cùng Windows`, `Chạy quyền admin (gõ được trong cửa sổ admin)` (chế độ admin), `Tự động cập nhật` kèm nút `Kiểm tra bản mới`, và `Chế độ phát triển` (mặc định tắt), bật lên thì ngay bên dưới có thêm nút `Mở thư mục cấu hình…` và dòng đo thời gian của hook. Khi có bản mới đang chờ, dòng dưới ô cập nhật ghi `Có bản mới vX.Y.Z.` kèm liên kết `Cập nhật tại đây` để cài bản đó hoặc mở trang tải về. Dòng cuối ghi tên tác giả (`TrioKey vX.Y.Z · © 2026 ved0el`) và liên kết `GitHub` tới repo này; bản chạy quyền admin hiện địa chỉ đó trong một hộp thoại thay vì mở trình duyệt. Thay đổi có hiệu lực ngay. Cửa sổ theo chế độ màu của ứng dụng trong Windows (sáng hoặc tối; trên Windows 11 thanh tiêu đề cùng màu với phần còn lại) và giữ giao diện hệ thống khi bật tương phản cao; đổi một trong hai khi cửa sổ đang mở thì cửa sổ đổi màu theo.
- **Telex**: trong một từ, `aa aw ee oo ow uw dd` → `â ă ê ô ơ ư đ` (`caan` → `cân`, `ddi` → `đi`); `s f r x j` → dấu thanh; `z` xoá dấu thanh. Phím dấu gõ ngay sau nguyên âm hay ở cuối từ đều được (`tieesng`, `tieengs` → `tiếng`). Chữ `w` đứng riêng hiện thành `ư` trong lúc gõ nhưng trở lại `w` khi từ kết thúc (`w␣` → `w`, rồi `⌫` → `ư`). Gõ một phím dấu thanh hai lần thì huỷ dấu (`mixx␣` → `mix`); từ tiếng Anh có chữ cái gõ đôi thì giữ nguyên như đã gõ.
- **Nhớ lần đổi.** Nếu bạn đổi một từ bằng một lần Backspace rồi gõ dấu cách, Tab hoặc Enter, TrioKey nhớ lần đổi đó ngay và dùng nó từ đó về sau. Gõ một chữ cái hay dấu câu ngay sau lần đổi thì không lưu gì. Đổi một từ đã nhớ trở lại bằng một lần Backspace sẽ sửa hoặc xoá mục đó. Có một ngoại lệ để bảo vệ chữ tiếng Việt: ở giữa một dòng đã nghiêng về tiếng Việt, cách đọc tiếng Anh đã nhớ không bao giờ thay từ tiếng Việt; nó chỉ nằm cách một lần Backspace, và giữ lần đổi đó thì mục đã nhớ vẫn như cũ. Mỗi bước dưới đây bắt đầu ở một dòng mới, theo đúng thứ tự:

  ```text
  phím gõ                  chữ hiện ra
  this␣⌫␣                  thí               (đổi rồi giữ: được nhớ)
  this␣                    thí
  this␣⌫␣                  this              (đổi lại rồi giữ: bị quên)
  this␣                    this
  songs␣⌫␣                 songs             (được nhớ là tiếng Anh)
  songs␣                   songs
  tooi thaasy songs␣       tôi thấy sóng     (dòng tiếng Việt giữ từ tiếng Việt...)
  tooi thaasy songs␣⌫      tôi thấy songs    (...chỉ cách một lần Backspace)
  ```

  Các từ đã nhớ nằm trong `%APPDATA%\triokey\learned.txt`, mỗi từ một dòng `phím<TAB>chữ`. Xoá một dòng để TrioKey quên từ đó; chỉ sửa file sau khi đã thoát TrioKey (`Thoát`), nếu không file sẽ bị ghi đè. Giữ tối đa 5.000 mục, và các mục không tự hết hạn. File được lưu khoảng 2 giây sau thay đổi cuối cùng, khi thoát và khi đăng xuất.
- **Ép một từ.** Muốn chọn hẳn một lần cho xong, hãy tạo `force_en.txt` (luôn là tiếng Anh) hoặc `force_vi.txt` (luôn là tiếng Việt) trong cùng thư mục. Mỗi dòng một từ, viết theo phím gõ (`this`, không phải `thí`); `#` mở đầu chú thích. Hai file này thắng các từ đã nhớ, lần đổi của từ bị ép không được nhớ, và một lần Backspace vẫn cho cách đọc kia nếu từ đó có. Hai file được đọc lúc khởi động, nên sửa xong hãy chạy lại TrioKey.
- **Cấu hình** nằm ở `%APPDATA%\triokey\config.txt` (các dòng `khoá = giá trị`). `dev_mode = true` là ô `Chế độ phát triển`.
- **Cập nhật.** Khi `Tự động cập nhật` bật (mặc định), TrioKey hỏi GitHub mỗi ngày một lần xem có bản mới hơn có chữ ký hay không (lần thử đầu khoảng một phút sau khi chạy; lần nào không nhận được trả lời, ví dụ khi mất mạng, thì một giờ sau thử lại). Bản thường tự tải về, kiểm chữ ký rồi thay file exe của chính nó; một thông báo hiện `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau`, và file exe cũ nằm cạnh đó dưới tên `triokey.old.exe` cho tới lần chạy sau. Bản chạy ở chế độ admin không bao giờ tự cài: nó tải file exe mới vào bộ nhớ, kiểm chữ ký, rồi hiện `Có bản X.Y.Z — bấm để cập nhật` ở thông báo và ở một dòng trong menu khay, và chỉ cập nhật khi bạn bấm vào một trong hai chỗ đó. Nếu TrioKey không ghi được vào thư mục của chính nó, hoặc file exe đã bị đổi sang tên khác `triokey.exe`, nó chỉ báo là có bản mới; bấm vào thông báo thì mở trang tải về. Một phiên bản chỉ được báo sau khi chữ ký của nó đã được kiểm.

## Gỡ bỏ

Các bước này viết theo mã nguồn; chưa từng được chạy trên máy thật. Nếu có bước không được, các lệnh gỡ tay (`schtasks /Delete`, `reg delete`) nằm ở mục cuối của [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md).

1. Nếu đã bật **chế độ admin** hoặc **khởi động cùng Windows**, hãy bỏ chúng trong Cài đặt theo đúng thứ tự sau:
   1. Bỏ chọn chế độ admin (`Chạy quyền admin (gõ được trong cửa sổ admin)`). Việc này xoá tác vụ đăng nhập `TrioKey` và bản sao trong `%ProgramFiles%\TrioKey`. Nếu làm từ chính bản admin đang chạy thì Windows xoá thư mục đó ở lần khởi động lại kế tiếp; hãy thoát bản đó và chạy bản thường rồi mới làm bước sau.
   2. Sau đó bỏ chọn `Khởi động cùng Windows`; ô này bị mờ khi chế độ admin còn bật. Tắt chế độ admin từ bản thường sẽ ghi lại giá trị Run nếu trước đó bạn đã bật, nên hãy xem lại ô này dù lâu rồi bạn không đụng tới. Bỏ chọn sẽ xoá giá trị `TrioKey` trong `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
2. Thoát từ menu ở khay (`Thoát`).
3. Xoá thư mục bạn đã giải nén, build hoặc chép TrioKey vào, và `%APPDATA%\triokey` nếu muốn bỏ luôn cấu hình, các file ép từ và các từ đã nhớ.

Nếu chưa từng bật hai tuỳ chọn trên thì bước 2 và 3 là toàn bộ việc gỡ. TrioKey không đăng ký gì khác: không layout bàn phím, không mục nào trong danh sách bàn phím của Windows, không gì dưới `HKCU\Keyboard Layout`.

## Riêng tư và bảo mật

Những gì nó làm:

- Cài một keyboard hook cấp thấp (`WH_KEYBOARD_LL`). Như mọi bộ gõ dùng hook, nó vì thế nhìn thấy mọi phím bạn bấm trong lúc đang chạy.
- Cài một hook chuột cấp thấp (`WH_MOUSE_LL`). Nó chỉ dùng các cú nhấp, để biết con trỏ có thể đã dời và IME có thể vừa được bật/tắt.
- Theo dõi việc đổi cửa sổ và đổi focus (hai WinEvent hook) và đọc thời điểm của thao tác nhập gần nhất.
- Gửi phím Backspace và chữ Unicode tới cửa sổ đang gõ bằng `SendInput`.
- Đọc layout bàn phím và trạng thái mở/đóng IME của cửa sổ đang gõ.
- Lưu trong `%APPDATA%\triokey`: `config.txt` (tạo ở lần chạy đầu, ghi lại khi bạn đổi một thiết lập), hai file ép từ do bạn tự tạo, và `learned.txt` (tạo ở lần chạy đầu, từ bản 0.2.0). `learned.txt` chỉ chứa những từ bạn đã đổi bằng Backspace rồi giữ lại: phím gõ và chữ hiện ra, ngoài ra không có gì, không thời gian, không thứ tự gõ.
- Đọc hai giá trị dưới `HKCU`: thiết lập sáng/tối của Windows (để chọn màu biểu tượng và màu cửa sổ cài đặt) và giá trị Run của chính nó.
- Nếu `Tự động cập nhật` đang bật: mỗi ngày một lần (lần thử đầu khoảng một phút sau khi chạy; lần nào không nhận được trả lời thì một giờ sau thử lại), lấy file chữ ký của bản phát hành mới nhất (vài trăm byte) từ `github.com`. Chỉ khi file đó nêu một phiên bản mới hơn, nó mới tải `triokey.exe` của chính bản phát hành ấy, kiểm chữ ký [minisign](https://jedisct1.github.io/minisign/) bằng khoá công khai nằm sẵn trong file exe đang chạy, rồi đặt file vào chỗ để dùng từ lần chạy sau. Mỗi file trong hai file này cần hai hoặc ba yêu cầu HTTPS, vì GitHub chuyển hướng lượt tải sang máy chủ chứa file phát hành. Yêu cầu chỉ mang một `User-Agent: TrioKey-updater` cố định và không gì khác: không có chữ bạn gõ, không có từ đã nhớ, không có mã nhận dạng máy hay người dùng, kể cả số phiên bản đang chạy cũng không. GitHub thấy địa chỉ IP của bạn, như mọi trang bạn tải file về. Thời điểm kiểm tra gần nhất được ghi trong `config.txt`; bản chạy ở chế độ admin chỉ giữ nó trong bộ nhớ, nên nó có thể kiểm tra thêm một lần sau mỗi lần chạy, rồi mỗi ngày một lần.
- Ở lần chạy đầu, nếu có thư mục `%APPDATA%\evj` (tên tạm của dự án trước khi công bố) thì chép `config.txt`, `force_vi.txt` và `force_en.txt` từ đó sang. Người dùng mới không có thư mục này.

Những gì nó không bao giờ làm:

- Ghi nhật ký gõ. Ngoài các từ đã đổi trong `learned.txt`, không phím hay nội dung gõ nào được ghi ra đĩa. Con số đo thời gian hiện trong Cài đặt (chế độ phát triển) chỉ nằm trong bộ nhớ.
- Kết nối tới nơi nào khác ngoài GitHub (`github.com` và máy chủ mà đường tải bản phát hành của nó chuyển hướng tới), hay kết nối vì việc gì khác ngoài việc kiểm tra cập nhật nói trên. Khi `Tự động cập nhật` tắt, TrioKey hoàn toàn không dùng mạng, trừ khi bạn bấm `Kiểm tra bản mới`; nút này kiểm tra một lần và, ở bản thường, cài luôn bản tìm được. Ở bản build không có khoá cập nhật, cả hai ô này bị mờ đi và nó không bao giờ kết nối. Không thu thập dữ liệu, không gửi báo cáo lỗi. Thư viện bên thứ ba mà nó phụ thuộc trực tiếp chỉ có ba crate `vi`, `windows-sys` và `minisign-verify`, cộng `embed-manifest` chỉ dùng lúc build (nó tạo manifest, rồi script build ghi manifest cùng biểu tượng vào file exe; không có mã nào của nó chạy).
- Đăng ký text service, thêm layout bàn phím hay đổi danh sách bàn phím của Windows.

Điều cần biết:

- **Mật khẩu.** Ô mật khẩu Win32 cổ điển (control Edit/RichEdit có `ES_PASSWORD`) được thiết kế để bỏ qua; chưa kiểm bằng tay. Ô mật khẩu trong trình duyệt và ứng dụng Electron thì không nhận ra được. Ở đó, một đoạn mật khẩu toàn chữ cái có thể tình cờ là Telex của một âm tiết tiếng Việt và bị đổi (`mas!x9` → `má!x9`). Vì vậy hãy tắt TrioKey bằng phím tắt khi gõ mật khẩu ở những ô như thế; biểu tượng khay cho biết đang ở trạng thái nào.
- **Chế độ admin** mặc định tắt và chưa từng chạy trên máy thật. Bật nó sẽ hiện một hộp UAC. Phần chạy quyền cao chép file exe vào `%ProgramFiles%\TrioKey` (chỉ admin ghi được, nên chương trình khác không thể đánh tráo exe để lấy quyền admin) và đăng ký một tác vụ chạy lúc đăng nhập với quyền cao nhất. Bản chạy quyền admin không chạy chương trình nào khác: nút mở thư mục cấu hình của nó (chế độ phát triển) chỉ hiện đường dẫn, không mở Explorer, và các liên kết trong cửa sổ cài đặt chỉ hiện địa chỉ, không mở trình duyệt. Nó không tạo gì trong `%APPDATA%`, và chỉ đọc `learned.txt`, ghi lại `learned.txt` hay `config.txt` khi file đó đã có sẵn dưới dạng file thường trong đúng thư mục `%APPDATA%\triokey` thật, không có liên kết nào trên đường dẫn; nếu không, thiết lập và các từ đã nhớ chỉ nằm trong bộ nhớ. Hãy bật chế độ admin từ một thư mục mà chương trình khác không ghi được: hộp UAC nâng quyền cho đúng file `triokey.exe` đang nằm trong thư mục đó lúc ấy. Chi tiết thiết kế: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), mục 8.
- **Cập nhật** chỉ được cài khi có chữ ký hợp lệ từ khoá cập nhật của dự án, ký cho đúng tên file `triokey.exe` và cho một phiên bản mới hơn bản đang chạy; mọi thứ khác đều bị từ chối, kể cả một bản cũ được đưa ra lại. Mọi sự tin cậy nằm ở chữ ký. Khoá do quy trình phát hành của dự án giữ, và quy trình này ký rồi phát hành một tag phiên bản đẩy lên `main` mà không cần ai duyệt tay: người lấy được khoá, hoặc đẩy được lên repo này (chỉ có chủ repo), đều đẩy được bản cập nhật tới mọi bản đã cài. Người chỉ thay được các file phát hành thì không. File exe không có chữ ký Authenticode, nên SmartScreen coi bản phát hành như mọi file tải về chưa ký khác. Ở chế độ admin, việc tải, kiểm chữ ký và thay file đều diễn ra trong `%ProgramFiles%\TrioKey`, và chỉ sau khi bạn bấm. Một TrioKey được chạy bằng "Run as administrator" từ thư mục khác thì không ghi gì vào thư mục đó: nó chỉ báo là có bản mới. Chi tiết: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), mục 9, và [docs/RELEASING.md](docs/RELEASING.md).
- **Phím do phần mềm khác gửi vào** (remote desktop, phần mềm đổi phím) dưới dạng phím bấm thông thường được xử lý như phím thật; chữ được gửi vào dưới dạng ký tự Unicode thì được để nguyên. Chưa thử với công cụ cụ thể nào. Đó cũng là lý do không được chạy hai bộ gõ cùng lúc. Từ bản 0.1.1, phím mà Windows đánh dấu là do một tiến trình có mức toàn vẹn (integrity level) thấp hơn gửi vào thì đi thẳng, không bị đụng tới, để chương trình không có quyền admin không khiến được một TrioKey quyền admin gõ chữ; vì thế khi TrioKey chạy quyền admin, phím do các công cụ như vậy gửi từ một tiến trình thường (AutoHotkey, bàn phím ảo) sẽ không thành tiếng Việt.

## Cách hoạt động

Hai crate:

- **`triokey-engine`**: Rust thuần, không gọi hệ điều hành. Nó giữ dãy phím gốc của từ đang gõ và, sau mỗi phím, tính lại từ đầu xem dãy phím ấy nên hiện thành gì, so với chữ đang có trên màn hình rồi trả về một lệnh sửa: xoá N ký tự, chèn một chuỗi. Giữa hai từ, lõi giữ rất ít: độ nghiêng của dòng, vài cờ về đoạn chữ đang gõ (có phải mã hay không), lần đổi bằng Backspace đang chờ và các từ đã nhớ. Lõi không đọc lại màn hình, nên chữ thật vẫn có thể khác ở những ô tự chèn gợi ý. Phần ghép Telex và đặt dấu thanh dùng crate [`vi`](https://github.com/ZeroX-DG/vi-rs); phần kiểm tra âm tiết, các bảng tần suất, các luật về mã và quyết định cho từng từ là của TrioKey. Lõi có bộ test riêng và một bài đo gõ chữ.
- **`triokey`**: ứng dụng khay cho Windows. Callback của hook hỏi lõi gõ và, khi chữ cần đổi, chặn phím rồi gửi Backspace cùng chữ mới bằng `SendInput` ngay trong callback, để không phím nào gõ sau chen vào giữa. Phím không làm đổi chữ thì đi thẳng, không bị đụng tới. Phím do chính TrioKey gửi được đánh dấu và bỏ qua. Thời gian p99 của callback được hiện trong cửa sổ cài đặt, ngay dưới ô `Chế độ phát triển`, khi ô đó được chọn. Lần đọc duy nhất tới nay, trên máy của tác giả với bản 0.2.0: 314 µs với phím đi thẳng và 5.600 µs với phím thay chữ, trên 502 phím.

Trạng thái IME được đọc từ cửa sổ IME mặc định của cửa sổ đang gõ (`ImmGetDefaultIMEWnd` + `WM_IME_CONTROL`): open status với tiếng Nhật, open status cùng conversion mode với tiếng Trung và tiếng Hàn. Một luồng riêng giữ cho giá trị này luôn mới, nên đường xử lý phím chỉ đọc một giá trị đã lưu sẵn.

Toàn bộ thiết kế, các luồng, các file dữ liệu, bản chạy quyền admin, cập nhật, quy trình phát hành và test (bằng tiếng Anh): [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Lộ trình

Không theo thứ tự cố định:

- Chạy danh sách gõ thử ([docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md)) trên bản hiện tại và ghi lại kết quả: tiếng Nhật, khay, cài đặt, khởi động cùng Windows, chế độ admin, phần tự cập nhật, các từ đã nhớ.
- Các ô tự chèn gợi ý (thanh địa chỉ trình duyệt, ô Excel).
- Gõ thử bằng tay với IME tiếng Trung và tiếng Hàn.
- VNI.
- Bản macOS và Linux dùng chung lõi gõ.

Không định làm: gõ tắt, nhớ chế độ theo ứng dụng, bảng mã cũ, tính năng cloud.

## Ghi nhận

- [`vi`](https://github.com/ZeroX-DG/vi-rs) (MIT): phần biến đổi Telex.
- [FineWeb-2](https://huggingface.co/datasets/HuggingFaceFW/fineweb-2) và [FineWeb](https://huggingface.co/datasets/HuggingFaceFW/fineweb) (ODC-By 1.0): số đếm từ làm nên các bảng tần suất tiếng Việt và tiếng Anh.
- [SCOWL](http://wordlist.aspell.net): danh sách từ điển tiếng Anh.
- [VieLotus-IM](https://github.com/stevedat/VieLotus-IM) (MIT): bộ test hồi quy công khai, được TrioKey chạy để canh không cho lõi gõ tụt lùi, và các danh sách từ dựa trên SCOWL làm đầu vào cho danh sách từ tiếng Anh của TrioKey.
- [VKey](https://github.com/phatMT97/VKey): cách nhường khi layout là CJK, thứ TrioKey vẫn quay về. Chỉ học hành vi.
- [Vikey](https://github.com/tonyminibot/Vikey): các quyết định đã công bố về phím dấu gõ lặp và về việc trả lại phím gốc. Chỉ học hành vi.
- [UniKey](https://www.unikey.org), EVKey và [OpenKey](https://github.com/tuyenvm/OpenKey): những bộ gõ lâu năm mà hầu hết người gõ tiếng Việt đã quen; TrioKey theo các phím Telex mà mọi người biết từ đó.

Không có mã nguồn, dữ liệu hay test nào lấy từ các dự án GPL.

Dòng bản quyền, nội dung giấy phép và chi tiết nằm ở [NOTICE.md](NOTICE.md). Đóng góp: [CONTRIBUTING.md](CONTRIBUTING.md).

## Giấy phép

[MIT](LICENSE).
