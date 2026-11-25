<p align="center"><img src="https://raw.githubusercontent.com/ved0el/triokey/main/assets/brand/icon-512.png" width="96" alt="TrioKey"></p>

# TrioKey

[English](README.md) · **Tiếng Việt**

[![CI](https://github.com/ved0el/triokey/actions/workflows/ci.yml/badge.svg)](https://github.com/ved0el/triokey/actions/workflows/ci.yml)

Bộ gõ tiếng Việt cho Windows, dành cho người ngày nào cũng gõ cả tiếng Anh, tiếng Việt và một trong ba thứ tiếng Trung, Nhật, Hàn (CJK). Khi gõ trộn hằng ngày, bạn không phải bật/tắt bộ gõ tiếng Việt: nút duy nhất cần bấm là nút bật/tắt của chính IME CJK.

<!-- TODO: chỗ đặt ảnh GIF minh hoạ (docs/assets/demo.gif): gõ Anh, Việt, Nhật trên cùng một dòng mà không chạm vào nút bật/tắt bộ gõ tiếng Việt -->

```text
phím gõ             chữ hiện ra
tieesng vieejt␣     tiếng việt
windows␣            windows
songs␣              songs        (từ hai cách đọc: mặc định là tiếng Anh)
this␣⌫              thí          (Backspace ngay sau từ sẽ đổi cách đọc)
```

Bản đầu 0.1.0: đã có bản tải về có chữ ký, nhưng nhiều phần của ứng dụng Windows, kể cả phần tự cập nhật đang bật sẵn, chưa từng chạy trên máy thật. Xin đọc mục [Tình trạng](#tình-trạng) trước.

## Vì sao có TrioKey

TrioKey sinh ra từ năm chỗ vướng hằng ngày. Hai chỗ đầu là lý do nó được viết; ba chỗ sau là những giới hạn nó tự đặt cho mình.

**1. Một ý định, hai lần bật/tắt.**
Trước đây: đang gõ tiếng Việt, muốn chen một từ tiếng Nhật. Bạn bật hiragana, rồi còn phải tắt chế độ tiếng Việt, nếu không Telex sẽ làm hỏng romaji. Lúc quay lại thì làm ngược cả hai.
Với TrioKey: bật hiragana, gõ, tắt hiragana. TrioKey đọc xem IME CJK có thực sự đang mở trong cửa sổ bạn đang gõ hay không rồi tự nhường. Khi IME đóng (direct input), nó gõ tiếng Việt, kể cả khi bàn phím đang ở layout Nhật.

**2. Chuyển qua lại giữa chế độ Anh và Việt.**
Trước đây: ở chế độ tiếng Việt, `terrible` hay `windows` gõ ra thành chữ vô nghĩa, nên cứ vài từ lại phải bấm chuyển.
Với TrioKey: không có chế độ nào cả. Mỗi từ được xét riêng. Dãy phím tạo thành một âm tiết tiếng Việt hợp lệ thì thành tiếng Việt; còn lại được trả về như các phím bạn đã gõ (ngoại lệ đã biết: phím dấu gõ lặp trong một từ nằm ngoài danh sách từ tiếng Anh bị gộp còn một, xem [Tình trạng](#tình-trạng)). Từ đúng theo cả hai cách (`songs` / `sóng`) được quyết định bằng danh sách từ và từ đứng trước; với từ có hai cách đọc, một lần Backspace là đổi sang cách kia.

**3. Bộ gõ đụng vào thiết lập bàn phím của Windows.**
TrioKey không thêm layout bàn phím, không đăng ký text service (TSF), không đụng vào danh sách bàn phím của Windows. Nó chỉ là một ứng dụng nhỏ ở khay hệ thống: một keyboard hook (kèm một hook chuột chỉ để biết có cú nhấp) và phần gửi chữ.

**4. Cửa sổ chạy quyền admin.**
Bộ gõ chạy quyền thường không gõ được vào cửa sổ quyền admin. TrioKey có chế độ admin tuỳ chọn: một lần đồng ý UAC, một tác vụ chạy lúc đăng nhập, và file exe được chép vào thư mục chỉ admin ghi được. (Đã viết xong nhưng chưa từng chạy trên máy thật; xem [Tình trạng](#tình-trạng).)

**5. Những tính năng bạn không cần.**
Không gõ tắt, không nhớ chế độ theo ứng dụng, không cloud, không thu thập dữ liệu, không lưu lại phím bạn gõ. Chỗ duy nhất dùng mạng là việc kiểm tra bản cập nhật có chữ ký trên GitHub, và bạn tắt được. Một cửa sổ cài đặt nhỏ và một file cấu hình dạng văn bản.

## Tình trạng

**Bản đầu 0.1.0. Xin đọc phần này trước khi dùng.**

- Bản 0.1.0 có trên [trang phát hành](https://github.com/ved0el/triokey/releases/latest), do CI build và ký. Bạn cũng có thể tự build từ mã nguồn.
- Phần tự cập nhật (tải bản có chữ ký từ trang phát hành GitHub của dự án) có trong file exe đã phát hành và trong bản build từ `main`, và mặc định bật, nhưng chưa từng chạy trên máy thật (AT10 trong [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) vẫn chưa làm). Nếu không muốn một phần tự cập nhật chưa được thử thay file exe của bạn, hãy bỏ chọn `Tự động cập nhật` trong Cài đặt.
- Mới thử trên Windows 11; Windows 10 chưa thử.
- Chỉ có Telex. Chưa có VNI.
- Tiếng Nhật: một bản dựng trước đây đã được gõ thử bằng tay với Microsoft IME trong Notepad và Brave, và đạt AT1, AT2 trong [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md) (các cột kết quả từng bước ở đó chưa được điền). Bản hiện tại chưa được gõ thử lại.
- Tiếng Trung và tiếng Hàn: mới chỉ có unit test cho luật quyết định (IME mở và đang ở chế độ bản ngữ), với giá trị giả. Các IME đó có báo đúng trạng thái hay không thì chưa ai thử.
- **Chưa từng chạy trên máy thật**, mới chỉ biên dịch và unit test ở chỗ làm được: biểu tượng khay và menu hiện tại, biểu tượng ứng dụng, các phím tắt, cửa sổ cài đặt, khởi động cùng Windows, chế độ admin, phần tự cập nhật, các bước gỡ bỏ và việc bỏ qua ô mật khẩu Win32. Các mục bên dưới mô tả chúng theo thiết kế.
- Giao diện (menu khay, tooltip, cửa sổ cài đặt, thông báo) chỉ có tiếng Việt.
- Phần tự nhận biết Anh/Việt đạt **92,0% (8.361 / 9.084)** trên bộ test hồi quy công khai của [VieLotus-IM](https://github.com/stevedat/VieLotus-IM), khi gõ mọi dòng mà không có ngữ cảnh (xem [docs/COMPARISON.vi.md](docs/COMPARISON.vi.md) để biết cách chạy này khác cách của chính VieLotus-IM ra sao). Các trường hợp sai đã biết được ghi ở [docs/decisions/0004-regression-baseline.md](docs/decisions/0004-regression-baseline.md) và [0005-tie-defaults.md](docs/decisions/0005-tie-defaults.md). Hai loại bạn sẽ gặp: từ nằm ngoài danh sách từ tiếng Anh mà có phím dấu gõ lặp (`s f r x j`) thì bị mất một chữ (`Larry` → `Lary`, `passwd` → `paswd`, `terraform` → `teraform`), và một số từ ngắn có nghĩa ở cả hai thứ tiếng bị đoán sai khi không có ngữ cảnh. Cả hai đều sửa được cho từng từ bằng `force_en.txt` / `force_vi.txt` (xem [Cách dùng](#cách-dùng)).
- Terminal không có xử lý riêng.
- Không nhận ra được ô mật khẩu trong trình duyệt và ứng dụng Electron (ô mật khẩu Win32 cổ điển thì được thiết kế để bỏ qua; chưa kiểm bằng tay). Xem [Riêng tư và bảo mật](#riêng-tư-và-bảo-mật).
- Các ô tự chèn gợi ý sau con trỏ (thanh địa chỉ trình duyệt, ô Excel) chưa có xử lý riêng.
- Một số ứng dụng giấu trạng thái IME. Ở đó TrioKey quay về cách "cứ layout CJK là nhường".
- macOS và Linux mới nằm trong kế hoạch, chưa bắt đầu.

## TrioKey gõ ra gì trong từng trường hợp

| Layout bàn phím của cửa sổ | Trạng thái IME CJK | Phím bạn gõ sẽ ra gì | Khay |
| --- | --- | --- | --- |
| Tiếng Anh, hoặc layout không phải CJK | (không liên quan) | Telex tiếng Việt; từ tiếng Anh giữ nguyên như đã gõ | `V` |
| Nhật | đóng (direct input) | Telex tiếng Việt; từ tiếng Anh giữ nguyên như đã gõ | `V` |
| Nhật | mở (hiragana, katakana…) | mọi phím đi thẳng tới IME, không bị đụng tới | `E` |
| Trung hoặc Hàn † | đóng, hoặc mở ở chế độ chữ Latin | Telex tiếng Việt; từ tiếng Anh giữ nguyên như đã gõ | `V` |
| Trung hoặc Hàn † | mở ở chế độ bản ngữ | mọi phím đi thẳng tới IME, không bị đụng tới | `E` |
| CJK, trong ứng dụng giấu trạng thái IME | không biết | TrioKey nhường | `E` |
| bất kỳ, sau khi bạn tắt TrioKey | (không liên quan) | mọi phím đi thẳng | `E` |

† Chưa thử với IME thật: đây là luật trong mã nguồn, mới chỉ được kiểm bằng unit test. Các dòng tiếng Nhật đã được gõ thử trên một bản dựng trước đây (xem [Tình trạng](#tình-trạng)).

TrioKey không đổi layout bàn phím hộ bạn. "Chỉ còn một nút" đúng khi bạn ngồi sẵn ở layout CJK và dùng nút bật/tắt của IME; nếu bạn đang ở layout tiếng Anh thì vẫn phải tự chuyển layout để sang tiếng Nhật, chỉ là không còn phải tắt tiếng Việt nữa.

### Ví dụ gõ

Mỗi dòng đều được bộ test của lõi gõ kiểm (`crates/triokey-engine/tests/engine.rs`). `␣` là phím cách, `⌫` là Backspace.

```text
phím gõ                    chữ hiện ra
tieesng vieejt␣            tiếng việt
windows␣                   windows
terrible␣                  terrible
DDuowngf␣                  Đường
baajt/taawst␣              bật/tắt
server cos␣                server có
my_var␣   obj.dir␣         my_var   obj.dir      (mã được để nguyên)
songs␣                     songs                 (từ hai cách đọc: mặc định là tiếng Anh)
tooi thaasy songs␣         tôi thấy sóng         (cùng dãy phím, đứng sau từ tiếng Việt)
this␣                      this
this␣⌫                     thí                   (một lần Backspace đổi lại cách đọc)
```

## So sánh

Tính đến tháng 11 năm 2026. Đây là bản ngắn; nguồn, mã commit và những điểm các dự án khác làm tốt hơn nằm ở [docs/COMPARISON.vi.md](docs/COMPARISON.vi.md).

| | TrioKey | [VKey](https://github.com/phatMT97/VKey) | [Vikey](https://github.com/tonyminibot/Vikey) (Windows) | [VieLotus-IM](https://github.com/stevedat/VieLotus-IM) (Windows) |
| --- | --- | --- | --- | --- |
| Nhường IME CJK | theo trạng thái mở/đóng của IME, từng cửa sổ | theo ngôn ngữ của layout (Nhật, Trung, Hàn) | theo ngôn ngữ của layout (mọi thứ tiếng khác Anh và Việt) | là một mục riêng trong danh sách bàn phím của Windows, đứng cạnh IME CJK |
| Từ tiếng Anh khi đang gõ tiếng Việt | xét từng từ, không có chế độ | chế độ V/E có phím chuyển; kiểm tra chính tả trả lại từ không phải tiếng Việt | tự động: sửa từ và chuyển chế độ | tự động |
| Cách gắn vào Windows | hook + gửi chữ, mặc định không đăng ký gì | hook + gửi chữ; có thêm thành phần TSF tuỳ chọn | hook + gửi chữ | text service TSF, được thêm vào danh sách bàn phím |
| Kiểu gõ | Telex | Telex, VNI, Simple Telex, gõ kết hợp | Telex, Simple Telex | Telex, VNI |
| Gõ tắt | không | có | không | có |
| Nền tảng | Windows 11 (Windows 10 chưa thử) | Windows 10/11 | macOS, Windows, Linux, iOS, Android | macOS, Windows, Linux, web, iOS, Android |
| Bản phát hành | 0.1.0 (GitHub, do CI ký) | có (winget) | đã ra các phiên bản đánh số (theo `DECISIONS.md` của dự án); bản tải cho Windows thì chưa kiểm | có (Microsoft Store) |
| Giấy phép | MIT | GPL-3.0; có một thư viện kiểm tra chính tả nguồn đóng, tuỳ chọn, nằm ngoài giấy phép này (bản Classic hoàn toàn mã nguồn mở) | GPL-3.0 | MIT |

Cả ba đều hoàn thiện hơn TrioKey và đã được phát hành tới người dùng; TrioKey học được từ cả ba. UniKey, EVKey và OpenKey, những lựa chọn quen thuộc lâu nay, xoay quanh một chế độ Việt/Anh có phím chuyển; bản so sánh này không đọc ba bộ gõ đó, nên những tính năng khác của chúng không được mô tả ở đây.

## Cài đặt

Chưa có bộ cài. Các bản phát hành do CI build và ký từ một tag phiên bản ([docs/RELEASING.md](docs/RELEASING.md)).

1. **Thoát mọi bộ gõ tiếng Việt khác trước** (UniKey, EVKey, OpenKey, VKey, Vikey…). Hai bộ gõ chạy cùng lúc sẽ sửa chữ của nhau.
2. Tải `triokey-X.Y.Z-windows-x64.zip` từ [bản phát hành mới nhất](https://github.com/ved0el/triokey/releases/latest) rồi giải nén vào một thư mục riêng, hoặc tự build từ mã nguồn (bên dưới).
3. Chạy `triokey.exe`. Không có cửa sổ nào hiện ra; ở khay hệ thống xuất hiện chữ `V` (có thể nằm sau mũi tên `^`).

### Build từ mã nguồn

1. Cài Rust. Chọn một trong hai:
   - [mise](https://mise.jdx.dev): chạy `mise install` trong thư mục repo (nó đọc `mise.toml`, file này chọn toolchain GNU bản stable nên không cần Visual Studio Build Tools), hoặc
   - [rustup](https://rustup.rs) với toolchain `stable-x86_64-pc-windows-gnu`. Toolchain MSVC mặc định không build được TrioKey: script build của nó ghi tài nguyên của file exe (biểu tượng và manifest) chỉ cho toolchain GNU.
2. Build:

   ```sh
   git clone https://github.com/ved0el/triokey
   cd triokey
   cargo build --release -p triokey        # dùng mise: mise x -- cargo build --release -p triokey
   ```

3. File exe nằm ở `target\release\triokey.exe`. Nó mang cùng khoá cập nhật với bản phát hành, nên cũng tự kiểm tra bản mới.

File exe chưa được ký số. SmartScreen hoặc phần mềm diệt virus có thể cảnh báo, như với nhiều chương trình dùng keyboard hook khác. File `triokey.exe.minisig` đăng kèm mỗi file exe đã phát hành có thể kiểm bằng tay với [minisign](https://jedisct1.github.io/minisign/) và khoá công khai trong `crates/triokey/update-pubkey.txt`; phần tự cập nhật có sẵn cũng kiểm đúng như thế.

## Cách dùng

Phần khay, phím tắt và cửa sổ cài đặt dưới đây được mô tả theo thiết kế; chúng chưa từng chạy trên máy thật (xem [Tình trạng](#tình-trạng)).

- **Biểu tượng khay.** `V` là TrioKey đang gõ tiếng Việt. `E` là phím đi thẳng, vì bạn đã tắt hoặc vì nó đang nhường IME CJK; di chuột lên để xem lý do (`Tiếng Anh — đã tắt` hoặc `Tiếng Anh — đang nhường IME Nhật`; với IME Trung và Hàn là `IME Trung`, `IME Hàn`).
- **Nhấp trái** vào biểu tượng để bật/tắt TrioKey. **Nhấp phải** để mở menu: `Tiếng Việt` (bật/tắt), `Cài đặt…`, `Thoát`.
- **Phím tắt** bật/tắt: Ctrl+Shift (mặc định; bấm rồi nhả, không kèm phím nào khác), Alt+Z, Ctrl+Space, hoặc `Không dùng`.
- **Cài đặt** (`Cài đặt…`): `Bật gõ tiếng Việt`, `Phím tắt`, `Kiểu bỏ dấu` (`hòa, thúy` hoặc `hoà, thuý`), `Tự giữ nguyên từ tiếng Anh`, `Nhường khi IME Nhật/Trung/Hàn đang bật`, `Khởi động cùng Windows`, `Chạy quyền admin (gõ được trong cửa sổ admin)` (chế độ admin), `Tự động cập nhật` kèm nút `Kiểm tra bản mới`, và nút `Mở thư mục cấu hình…`. Thay đổi có hiệu lực ngay.
- **Cập nhật.** Khi `Tự động cập nhật` bật (mặc định), TrioKey hỏi GitHub mỗi ngày một lần xem có bản mới hơn có chữ ký hay không (lần thử đầu khoảng một phút sau khi chạy; lần nào không nhận được trả lời, ví dụ khi mất mạng, thì một giờ sau thử lại). Bản thường tự tải về, kiểm chữ ký rồi thay file exe của chính nó; một thông báo hiện `Đã cập nhật lên X.Y.Z, có hiệu lực lần chạy sau`, và file exe cũ nằm cạnh đó dưới tên `triokey.old.exe` cho tới lần chạy sau. Bản chạy ở chế độ admin không bao giờ tự cài: nó tải file exe mới vào bộ nhớ, kiểm chữ ký, rồi hiện `Có bản X.Y.Z — bấm để cập nhật` ở thông báo và ở một dòng trong menu khay, và chỉ cập nhật khi bạn bấm vào một trong hai chỗ đó. Nếu TrioKey không ghi được vào thư mục của chính nó, hoặc file exe đã bị đổi sang tên khác `triokey.exe`, nó chỉ báo là có bản mới và chỉ chỗ tải. Một phiên bản chỉ được báo sau khi chữ ký của nó đã được kiểm. Chưa chạy trên máy thật.
- **Cấu hình** nằm ở `%APPDATA%\triokey\config.txt` (các dòng `khoá = giá trị`).
- **Ép một từ.** Tạo `force_en.txt` (luôn là tiếng Anh) hoặc `force_vi.txt` (luôn là tiếng Việt) trong cùng thư mục. Mỗi dòng một từ, viết theo phím gõ (`this`, không phải `thí`); `#` mở đầu chú thích. Hai file được đọc lúc khởi động, nên sửa xong hãy chạy lại TrioKey.
- **Backspace đổi lại cách đọc.** Ngay sau khi một từ được chốt (dấu cách, dấu câu), lần Backspace đầu tiên đổi cách đọc của từ có hai cách đọc (`this␣⌫` → `thí`) hoặc của từ vừa bị luật phím lặp sửa (`mixx␣` → `mix`, rồi `⌫` → `mixx`). Backspace lần nữa mới xoá như thường. Với mọi từ khác, Backspace là xoá lùi bình thường.
- **Telex**: trong một từ, `aa aw ee oo ow uw dd` → `â ă ê ô ơ ư đ` (`caan` → `cân`, `ddi` → `đi`); gõ riêng thì `aa`, `aw`, `dd` vẫn giữ nguyên như đã gõ; `s f r x j` → dấu thanh; `z` xoá dấu thanh; `w` đứng riêng → `ư`. Phím dấu gõ ngay sau nguyên âm hay ở cuối từ đều được (`tieesng`, `tieengs` → `tiếng`).

## Gỡ bỏ

Các bước này viết theo mã nguồn; chưa từng được chạy trên máy thật. Nếu có bước không được, các lệnh gỡ tay (`schtasks /Delete`, `reg delete`) nằm ở mục cuối của [docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md).

1. Nếu đã bật **chế độ admin** hoặc **khởi động cùng Windows**, hãy bỏ chúng trong Cài đặt theo đúng thứ tự sau:
   1. Bỏ chọn chế độ admin (`Chạy quyền admin (gõ được trong cửa sổ admin)`). Việc này xoá tác vụ đăng nhập `TrioKey` và bản sao trong `%ProgramFiles%\TrioKey`. Nếu làm từ chính bản admin đang chạy thì Windows xoá thư mục đó ở lần khởi động lại kế tiếp; hãy thoát bản đó và chạy bản thường rồi mới làm bước sau.
   2. Sau đó bỏ chọn `Khởi động cùng Windows`; ô này bị mờ khi chế độ admin còn bật. Tắt chế độ admin từ bản thường sẽ ghi lại giá trị Run nếu trước đó bạn đã bật, nên hãy xem lại ô này dù lâu rồi bạn không đụng tới. Bỏ chọn sẽ xoá giá trị `TrioKey` trong `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
2. Thoát từ menu ở khay (`Thoát`).
3. Xoá thư mục bạn đã giải nén, build hoặc chép TrioKey vào, và `%APPDATA%\triokey` nếu muốn bỏ luôn cấu hình.

Nếu chưa từng bật hai tuỳ chọn trên thì bước 2 và 3 là toàn bộ việc gỡ. TrioKey không đăng ký gì khác: không layout bàn phím, không mục nào trong danh sách bàn phím của Windows, không gì dưới `HKCU\Keyboard Layout`.

## Riêng tư và bảo mật

Những gì nó làm:

- Cài một keyboard hook cấp thấp (`WH_KEYBOARD_LL`). Như mọi bộ gõ dùng hook, nó vì thế nhìn thấy mọi phím bạn bấm trong lúc đang chạy.
- Cài một hook chuột cấp thấp (`WH_MOUSE_LL`). Nó chỉ dùng các cú nhấp, để biết con trỏ có thể đã dời và IME có thể vừa được bật/tắt.
- Theo dõi việc đổi cửa sổ và đổi focus (hai WinEvent hook) và đọc thời điểm của thao tác nhập gần nhất.
- Gửi phím Backspace và chữ Unicode tới cửa sổ đang gõ bằng `SendInput`.
- Đọc layout bàn phím và trạng thái mở/đóng IME của cửa sổ đang gõ.
- Đọc file cấu hình và hai file ép từ, tạo `config.txt` ở lần chạy đầu và ghi lại nó khi bạn đổi một thiết lập.
- Đọc hai giá trị dưới `HKCU`: thiết lập sáng/tối của Windows (để chọn màu biểu tượng) và giá trị Run của chính nó.
- Nếu `Tự động cập nhật` đang bật: mỗi ngày một lần (lần thử đầu khoảng một phút sau khi chạy; lần nào không nhận được trả lời, ví dụ khi mất mạng, thì một giờ sau thử lại), lấy file chữ ký của bản phát hành mới nhất (vài trăm byte) từ `github.com`. Chỉ khi file đó nêu một phiên bản mới hơn, nó mới tải `triokey.exe` của chính bản phát hành ấy, kiểm chữ ký [minisign](https://jedisct1.github.io/minisign/) bằng khoá công khai nằm sẵn trong file exe đang chạy, rồi đặt file vào chỗ để dùng từ lần chạy sau. Mỗi file trong hai file này cần hai hoặc ba yêu cầu HTTPS, vì GitHub chuyển hướng lượt tải sang máy chủ chứa file phát hành. Yêu cầu chỉ mang một `User-Agent: TrioKey-updater` cố định và không gì khác: không có chữ bạn gõ, không có mã nhận dạng máy hay người dùng, kể cả số phiên bản đang chạy cũng không. GitHub thấy địa chỉ IP của bạn, như mọi trang bạn tải file về. Thời điểm kiểm tra gần nhất được ghi trong `config.txt`; bản chạy ở chế độ admin chỉ giữ nó trong bộ nhớ, nên nó có thể kiểm tra thêm một lần sau mỗi lần chạy, rồi mỗi ngày một lần.
- Ở lần chạy đầu, nếu có thư mục `%APPDATA%\evj` (tên tạm của dự án trước khi công bố) thì chép `config.txt`, `force_vi.txt` và `force_en.txt` từ đó sang. Người dùng mới không có thư mục này.

Những gì nó không bao giờ làm:

- Ghi phím hay nội dung gõ ra đĩa. Không có nhật ký gõ. Con số đo thời gian hiện trong Cài đặt chỉ nằm trong bộ nhớ.
- Kết nối tới nơi nào khác ngoài GitHub (`github.com` và máy chủ mà đường tải bản phát hành của nó chuyển hướng tới), hay kết nối vì việc gì khác ngoài việc kiểm tra cập nhật nói trên. Khi `Tự động cập nhật` tắt, TrioKey hoàn toàn không dùng mạng, trừ khi bạn bấm `Kiểm tra bản mới`; nút này kiểm tra một lần và, ở bản thường, cài luôn bản tìm được. Ở bản build không có khoá cập nhật, cả hai ô này bị mờ đi và nó không bao giờ kết nối. Không thu thập dữ liệu, không gửi báo cáo lỗi. Thư viện bên thứ ba mà nó phụ thuộc trực tiếp chỉ có ba crate `vi`, `windows-sys` và `minisign-verify`, cộng `embed-manifest` chỉ dùng lúc build (nó tạo manifest, rồi script build ghi manifest cùng biểu tượng vào file exe; không có mã nào của nó chạy).
- Đăng ký text service, thêm layout bàn phím hay đổi danh sách bàn phím của Windows.

Điều cần biết:

- **Mật khẩu.** Ô mật khẩu Win32 cổ điển (control Edit/RichEdit có `ES_PASSWORD`) được thiết kế để bỏ qua; chưa kiểm bằng tay. Ô mật khẩu trong trình duyệt và ứng dụng Electron thì không nhận ra được. Ở đó, một đoạn của mật khẩu toàn chữ cái, viết hoa/thường bình thường và đứng trước chữ số hoặc dấu `_` đầu tiên có thể bị đổi theo hai cách: nó tình cờ là Telex của một âm tiết tiếng Việt (đoạn `mas` trong `mas!x9` thành `má`), hoặc nó có phím dấu gõ lặp (`s f r x j`) và không nằm trong danh sách từ tiếng Anh, khi đó nó mất một chữ (`Tess!` → `Tes!`, `passwd!` → `paswd!`). Vì vậy hãy tắt TrioKey bằng phím tắt khi gõ mật khẩu ở những ô như thế; biểu tượng khay cho biết đang ở trạng thái nào.
- **Chế độ admin** mặc định tắt. Nó chưa từng chạy trên máy thật; phần dưới đây là thiết kế. Bật nó sẽ hiện một hộp UAC. Phần chạy quyền cao chép file exe vào `%ProgramFiles%\TrioKey` (chỉ admin ghi được, nên chương trình khác không thể đánh tráo exe để lấy quyền admin) và đăng ký một tác vụ chạy lúc đăng nhập với quyền cao nhất. Bản chạy quyền admin chỉ đọc các file dữ liệu có tên cố định và không chạy chương trình nào khác: nút mở thư mục cấu hình của nó chỉ hiện đường dẫn, không mở Explorer. Nó chỉ ghi lại `config.txt` khi file đó đã có sẵn và cả file lẫn thư mục đều không phải là liên kết, và không tạo gì trong `%APPDATA%`. Hãy bật chế độ admin từ một thư mục mà chương trình khác không ghi được: hộp UAC nâng quyền cho đúng file `triokey.exe` đang nằm trong thư mục đó lúc ấy.
- **Cập nhật** chỉ được cài khi có chữ ký hợp lệ từ khoá cập nhật của dự án, ký cho đúng tên file `triokey.exe` và cho một phiên bản mới hơn bản đang chạy; mọi thứ khác đều bị từ chối, kể cả một bản cũ được đưa ra lại. Mọi sự tin cậy nằm ở chữ ký. Khoá do quy trình phát hành của dự án giữ, và quy trình này ký rồi phát hành một tag phiên bản đẩy lên `main` mà không cần ai duyệt tay: người lấy được khoá, hoặc đẩy được lên repo này (chỉ có chủ repo), đều đẩy được bản cập nhật tới mọi bản đã cài. Người chỉ thay được các file phát hành thì không. File exe không có chữ ký Authenticode, nên SmartScreen coi bản phát hành như mọi file tải về chưa ký khác. Ở chế độ admin, việc tải, kiểm chữ ký và thay file đều diễn ra trong `%ProgramFiles%\TrioKey`, và chỉ sau khi bạn bấm. Một TrioKey được chạy bằng "Run as administrator" từ thư mục khác thì không ghi gì vào thư mục đó: nó chỉ báo là có bản mới. Chưa chạy trên máy thật; ghi chú thiết kế ở [docs/decisions/0007-auto-update.md](docs/decisions/0007-auto-update.md).
- **Phím do phần mềm khác gửi vào** (remote desktop, phần mềm đổi phím) dưới dạng phím bấm thông thường được xử lý như phím thật; chữ được gửi vào dưới dạng ký tự Unicode thì được để nguyên. Chưa thử với công cụ cụ thể nào. Đó cũng là lý do không được chạy hai bộ gõ cùng lúc. Sau bản 0.1.0 (có trong bản build từ `main`, chưa có trong bản phát hành nào, chưa chạy trên máy thật): phím mà Windows đánh dấu là do một tiến trình có mức toàn vẹn (integrity level) thấp hơn gửi vào thì đi thẳng, không bị đụng tới, để chương trình không có quyền admin không khiến được một TrioKey quyền admin gõ chữ. Vì thế khi TrioKey chạy quyền admin (chế độ admin), phím do các công cụ như vậy gửi từ một tiến trình thường, ví dụ AutoHotkey hay bàn phím ảo, sẽ không thành tiếng Việt.

## Cách hoạt động

Hai crate:

- **`triokey-engine`**: Rust thuần, không gọi hệ điều hành. Nó giữ dãy phím gốc của từ đang gõ và, sau mỗi phím, tính lại từ đầu xem dãy phím ấy nên hiện thành gì, so với chữ đang có trên màn hình rồi trả về một lệnh sửa: xoá N ký tự, chèn một chuỗi. Ngoài vài cờ (mã, giữ nguyên, ngữ cảnh từ đứng trước), lõi không giữ trạng thái nào tích luỹ qua từng phím, nên không bị lệch dần so với mô hình màn hình của chính nó. Lõi không đọc lại màn hình, nên chữ thật vẫn có thể khác ở những ô tự chèn gợi ý. Phần ghép Telex và đặt dấu thanh dùng crate [`vi`](https://github.com/ZeroX-DG/vi-rs); phần kiểm tra âm tiết tiếng Việt, danh sách từ tiếng Anh, các luật về mã và quyết định cho từng từ là của TrioKey. Lõi có bộ test riêng.
- **`triokey`**: ứng dụng khay cho Windows. Callback của hook hỏi lõi gõ và, khi chữ cần đổi, chặn phím rồi gửi Backspace cùng chữ mới bằng `SendInput` ngay trong callback, để không phím nào gõ sau chen vào giữa. Phím không làm đổi chữ thì đi thẳng, không bị đụng tới. Phím do chính TrioKey gửi được đánh dấu và bỏ qua. Thời gian p99 của callback được hiện ở cuối cửa sổ cài đặt; chưa có con số nào được công bố.

Trạng thái IME được đọc từ cửa sổ IME mặc định của cửa sổ đang gõ (`ImmGetDefaultIMEWnd` + `WM_IME_CONTROL`): open status với tiếng Nhật, open status cùng conversion mode với tiếng Trung và tiếng Hàn. Một luồng riêng giữ cho giá trị này luôn mới (khi đổi focus; sau các phím không gõ ra chữ như phím F và phím Kanji/IME, sau phím tắt và sau cú nhấp chuột; và theo nhịp 100 ms), nên đường xử lý phím chỉ đọc một giá trị đã lưu sẵn. Nếu IME được bật/tắt bằng một phím bổ trợ bấm riêng như Shift thì đến nhịp kế tiếp TrioKey mới thấy. Sau một thao tác có thể đã bật/tắt IME, phím đầu tiên có thể phải chờ khoảng 30 ms để lấy giá trị mới.

Ghi chú thiết kế: [docs/decisions/](docs/decisions/).

## Lộ trình

Không theo thứ tự cố định:

- Chạy danh sách gõ thử ([docs/HANDS_ON_TEST.md](docs/HANDS_ON_TEST.md)) trên bản hiện tại và ghi lại kết quả: tiếng Nhật, khay, cài đặt, khởi động cùng Windows, chế độ admin, phần tự cập nhật.
- Các ô tự chèn gợi ý (thanh địa chỉ trình duyệt, ô Excel).
- Gõ thử bằng tay với IME tiếng Trung và tiếng Hàn.
- Quyết định tốt hơn cho những từ đúng ở cả hai thứ tiếng.
- VNI.
- Bản macOS và Linux dùng chung lõi gõ.

Không định làm: gõ tắt, nhớ chế độ theo ứng dụng, bảng mã cũ, tính năng cloud.

## Ghi nhận

- [`vi`](https://github.com/ZeroX-DG/vi-rs) (MIT): phần biến đổi Telex.
- [SCOWL](http://wordlist.aspell.net): danh sách từ tiếng Anh.
- [VieLotus-IM](https://github.com/stevedat/VieLotus-IM) (MIT): bộ test hồi quy công khai của dự án này là thước đo của TrioKey, và các danh sách từ của nó là đầu vào cho từ điển của TrioKey.
- [VKey](https://github.com/phatMT97/VKey): cách nhường khi layout là CJK, thứ TrioKey vẫn quay về. Chỉ học hành vi.
- [Vikey](https://github.com/tonyminibot/Vikey): các quyết định đã công bố về phím dấu gõ lặp và về việc trả lại phím gốc. Chỉ học hành vi.
- [UniKey](https://www.unikey.org), EVKey và [OpenKey](https://github.com/tuyenvm/OpenKey): những bộ gõ lâu năm mà hầu hết người gõ tiếng Việt đã quen; TrioKey theo các phím Telex mà mọi người biết từ đó.

Không có mã nguồn, dữ liệu hay test nào lấy từ các dự án GPL.

Dòng bản quyền, nội dung giấy phép và chi tiết nằm ở [NOTICE.md](NOTICE.md).

## Giấy phép

[MIT](LICENSE).
