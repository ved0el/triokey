# TrioKey và các bộ gõ tương tự

[English](COMPARISON.md) · **Tiếng Việt**

Tính đến tháng 11 năm 2026.

Trang này so sánh TrioKey với ba bộ gõ tiếng Việt mã nguồn mở mà TrioKey đã học hỏi. Trang này do chính phía TrioKey viết, nên bạn hãy lưu ý điều đó khi đọc. Mỗi nhận định về dự án khác đều ghi rõ file đã đọc, tại commit liệt kê bên dưới. Chỗ nào chưa kiểm thì ô ghi "chưa kiểm" chứ không đoán. Nếu thấy sai, xin mở issue để sửa.

Cả ba dự án đều hoàn thiện hơn TrioKey và đã được phát hành: VKey có trên winget, VieLotus-IM có trên Microsoft Store và có gói cho Debian, còn Vikey ra các phiên bản đánh số kèm bộ cài và phần tự cập nhật. TrioKey là bản thử, chưa có bản dựng sẵn, và nhiều phần của ứng dụng Windows chưa từng chạy trên máy thật (xem mục Tình trạng trong [README](../README.vi.md)).

## Đã đọc những gì

| Dự án | Repo | Commit đã đọc | Giấy phép |
| --- | --- | --- | --- |
| VKey | https://github.com/phatMT97/VKey | `da22eab` (2026-11-19) | GPL-3.0 (`LICENSE`), kèm ngoại lệ nêu trong `README.md` của dự án (xem mục Nguồn) |
| Vikey | https://github.com/tonyminibot/Vikey | `c9007fe` (2026-11-21) | GPL-3.0 (`LICENSE`, `README.md`) |
| VieLotus-IM | https://github.com/stevedat/VieLotus-IM | `3519761` (2026-11-25) | MIT (`LICENSE`) |

Chỉ đọc repo. Trang phát hành, kho ứng dụng và danh sách issue thì không đọc. Khi làm bản so sánh này, mình không cài hay chạy chương trình nào trong ba chương trình trên, nên mọi thứ dưới đây mô tả hành vi theo tài liệu hoặc theo mã nguồn, không phải hành vi quan sát được.

UniKey, EVKey và OpenKey không được so sánh. Nói chung, đó là những lựa chọn quen thuộc lâu nay, xoay quanh một chế độ Việt/Anh có phím chuyển. Trang này không đọc ba bộ gõ đó, nên những tính năng khác của chúng không được mô tả ở đây.

## Bảng so sánh

| | TrioKey | VKey | Vikey (bản Windows) | VieLotus-IM (bản Windows) |
| --- | --- | --- | --- | --- |
| Ngôn ngữ lập trình | Rust | C++20 | lõi gõ và ứng dụng Windows bằng C++ (ứng dụng macOS bằng Swift) | lõi gõ và phần Windows bằng C++20 (lõi gõ macOS bằng Swift) |
| Giấy phép | MIT | GPL-3.0; có một thư viện kiểm tra chính tả nguồn đóng, tuỳ chọn, nằm ngoài giấy phép này (bản Classic hoàn toàn mã nguồn mở). Viết lại từ một fork của OpenKey | GPL-3.0, dựa trên OpenKey | MIT |
| Cách gắn vào Windows | keyboard hook cấp thấp (kèm một hook chuột để biết cú nhấp) + `SendInput`; mặc định không đăng ký gì | keyboard hook là đường chính; có thành phần TSF tuỳ chọn, đăng ký như một bộ gõ của Windows | keyboard hook cấp thấp + `SendInput` | text service TSF, đăng ký dưới tiếng Việt trong danh sách bàn phím của Windows |
| Gõ bên cạnh IME CJK | đọc trạng thái mở/đóng của IME theo từng cửa sổ; gõ tiếng Việt khi IME ở direct input dưới layout CJK (mới gõ thử bằng tay với tiếng Nhật, trên một bản dựng trước đây) | tự chuyển sang chế độ tiếng Anh khi ngôn ngữ của layout là Nhật, Trung hoặc Hàn; rời layout đó thì về lại tiếng Việt (có tuỳ chọn bật/tắt; trừ ứng dụng đã khoá ở tiếng Việt và ứng dụng bị loại trừ) | cho phím đi thẳng khi ngôn ngữ của layout không phải Anh hay Việt (có tuỳ chọn bật/tắt) | là một mục riêng trong danh sách bàn phím của Windows; cách nó chung sống với IME CJK thì chưa kiểm |
| Từ tiếng Anh trong câu tiếng Việt | không có chế độ; xét từng từ | chế độ V/E với phím chuyển và nhớ theo ứng dụng; kiểm tra chính tả trả lại phím gốc cho từ không phải tiếng Việt | tự động: sửa từ và chuyển chế độ; hoàn tác bằng Backspace (hai lần nếu từ được sửa khi gõ dấu cách) | tự xử lý song ngữ |
| Từ đúng ở cả hai thứ tiếng | danh sách từ + từ đứng trước; Backspace đổi lại | chưa kiểm | thống kê từ, gồm cả cặp từ; các từ đứng cạnh quyết định | có xử lý; 369/369 (lõi C++; 362/369 ở lõi Swift) khi harness của dự án chạy lại các từ trùng của cùng bộ test mà không có ngữ cảnh |
| Kiểu gõ | Telex | Telex, VNI, Simple Telex, kết hợp Telex+VNI | Telex, Simple Telex 1 và 2; không có VNI | Telex, VNI |
| Bảng mã cũ | không | có (TCVN3, VNI Windows…) | không (đã bỏ) | chưa kiểm |
| Gõ tắt | không | có | không (đã bỏ) | có |
| Nhớ theo ứng dụng | không | có; có luật theo tên miền qua một extension trình duyệt (thử nghiệm) | có, theo ứng dụng và theo trang web | chưa kiểm |
| Cửa sổ quyền admin | chế độ admin tuỳ chọn: tác vụ lúc đăng nhập, exe trong thư mục chỉ admin ghi được (đã viết, chưa chạy trên máy thật) | chạy được bằng quyền Administrator | không tìm thấy trong mã nguồn Windows | chưa kiểm |
| Dùng mạng | chỉ kiểm tra bản cập nhật có chữ ký trên GitHub, tắt được (đã viết, chưa chạy) | kiểm tra cập nhật | tự cập nhật; báo cáo lỗi có hỏi trước | chưa kiểm với bản Windows |
| Nền tảng | Windows 11 (Windows 10 chưa thử) | Windows 10/11 | macOS, Windows, Linux, iOS, Android | macOS, Windows, Linux, web (WASM), iOS, Android |
| Phân phối | build từ mã nguồn; đã có quy trình phát hành có chữ ký kèm phần tự cập nhật, chưa có bản phát hành nào | bản phát hành, winget | có script bộ cài và phần tự cập nhật trong repo; có phiên bản đánh số (`DECISIONS.md` mục 61 nhắc bản 1.0.33); trang phát hành thì chưa đọc | bản phát hành, Microsoft Store |

## TrioKey khác ở đâu

- **Nó đọc trạng thái của IME, không chỉ layout.** Dưới layout Nhật mà IME đóng, TrioKey gõ tiếng Việt; bật hiragana thì nó nhường. Khi bật tuỳ chọn tương ứng, VKey và Vikey nhường mỗi khi layout CJK đang hoạt động, bất kể IME có đang soạn chữ hay không (VKey trừ các ứng dụng đã khoá ở tiếng Việt). Ở ứng dụng giấu trạng thái IME, TrioKey cũng làm y như hai dự án đó.
- **Không có chế độ Việt/Anh.** Vẫn có nút bật/tắt, nhưng gõ trộn bình thường không cần tới nó. Vikey và VieLotus-IM cũng tự xử lý tiếng Anh; điểm khác là TrioKey không có chế độ nào để chuyển.
- **Mặc định không đăng ký gì vào Windows**: không TSF, không layout, không mục nào trong danh sách bàn phím của Windows. Tuỳ chọn khởi động cùng Windows thêm một giá trị Run; tuỳ chọn chế độ admin thêm một tác vụ và một thư mục.
- **Cố ý nhỏ**: một cửa sổ cài đặt, cấu hình dạng văn bản, và không có mã nào dùng mạng ngoài phần kiểm tra cập nhật (hai địa chỉ GitHub cố định, không gửi gì đi).
- **MIT, lõi gõ là một crate không gọi hệ điều hành**, nên dùng lại được ở nơi khác.

## Các dự án khác tốt hơn ở đâu

- **Độ hoàn thiện và việc phát hành.** VKey có trên winget, VieLotus-IM có trên Microsoft Store và có gói cho Debian, còn Vikey ra các phiên bản đánh số kèm bộ cài và phần tự cập nhật. TrioKey mới chỉ được chính tác giả gõ thử, trên một bản dựng trước đây, và chưa có bản dựng sẵn.
- **Chất lượng tự chuyển Anh/Việt.** Vikey quyết định từ trùng bằng thống kê từ và cặp từ dựng từ các kho văn bản lớn, và tinh chỉnh trên một tập test khoá riêng (`DECISIONS.md` mục 29, 30, 34; `ACKNOWLEDGEMENTS.md`). VieLotus-IM báo 9.091/9.091 trên bộ test của mình theo cách harness của dự án chạy (có mồi ngữ cảnh tiếng Anh cho từ trùng) và 362/369 (Swift) hoặc 369/369 (C++) trên các từ trùng của bộ test đó khi không có ngữ cảnh; TrioKey đạt 92,0% khi chạy mọi dòng không ngữ cảnh. TrioKey chỉ dùng một danh sách từ, một bảng mặc định chỉnh tay và từ đứng trước.
- **Kiểu gõ.** VKey và VieLotus-IM có VNI; VKey còn có Simple Telex, chế độ kết hợp và các bảng mã cũ. Vikey có các biến thể Simple Telex. TrioKey chỉ có Telex.
- **Gõ tắt.** VKey và VieLotus-IM có. TrioKey không có và không định làm.
- **Hành vi theo ứng dụng.** VKey và Vikey nhớ chế độ theo ứng dụng; Vikey còn nhớ theo trang web, và VKey có luật theo tên miền qua một extension trình duyệt (thử nghiệm). VKey còn có cấu hình riêng cho từng ứng dụng, loại trừ ứng dụng và chế độ game. TrioKey không có những thứ này.
- **Nền tảng.** Vikey và VieLotus-IM có macOS, Linux và di động; VKey chạy cả Windows 10 lẫn 11. TrioKey mới chỉ được thử trên Windows 11.
- **Đường TSF.** Một text service sửa chữ thông qua chính ứng dụng thay vì gửi phím Backspace, nên tránh được một lớp lỗi mà các bộ gõ dùng hook gặp ở ô tự chèn gợi ý và ở một số ứng dụng. VieLotus-IM xây trên TSF, còn VKey cho chọn TSF theo từng ứng dụng. TrioKey chọn không đăng ký text service; cái giá là những ô như vậy cần xử lý riêng mà TrioKey chưa có.
- **Ký số và đóng gói.** VKey có trên winget với gói cập nhật do dự án ký; VieLotus-IM có trên Microsoft Store. TrioKey chưa có gói cài và chưa có bản phát hành; phần tự cập nhật của nó chỉ nhận file được ký bằng khoá minisign của dự án, nhưng file exe không có chữ ký Authenticode nên Windows vẫn coi là chưa ký.
- **Ô mật khẩu và terminal.** TrioKey mới chỉ được thiết kế để bỏ qua ô mật khẩu Win32 cổ điển (chưa kiểm bằng tay) và không có xử lý riêng cho terminal. Ba dự án kia xử lý hai chỗ này ra sao thì chưa kiểm, nên rất có thể họ làm tốt hơn.
- **Ngôn ngữ giao diện.** Giao diện của TrioKey chỉ có tiếng Việt.

## TrioKey đã lấy gì từ họ

- Từ các quyết định Vikey đã công bố: hành vi của phím dấu gõ lặp và của việc trả lại phím gốc. Chỉ lấy hành vi; không lấy mã nguồn, dữ liệu hay test.
- Từ VieLotus-IM (MIT): bộ test hồi quy, danh sách từ dựa trên SCOWL và một phần từ điển, có ghi nguồn kèm nội dung giấy phép trong [NOTICE.md](../NOTICE.md).
- Từ VKey: hành vi nền "cứ layout CJK là nhường", thứ TrioKey vẫn quay về khi một ứng dụng giấu trạng thái IME. Chỉ lấy hành vi, đọc từ README và mã nguồn của dự án; không lấy mã.

## TrioKey trên bộ test của VieLotus-IM

TrioKey chạy `test_suite.csv` của VieLotus-IM (chép nguyên từ commit `3519761`, MIT) để lấy số liệu tham khảo, không phải bài test bắt buộc phải đạt. Hai kết quả dưới đây lấy từ cùng một file nhưng **không được đo theo cùng một cách**, nên không so ngang được:

| | Kết quả | Cách chạy các dòng |
| --- | --- | --- |
| VieLotus-IM (số liệu của chính dự án) | 9.091 / 9.091 | harness của dự án: 1.033 dòng `ambiguous_needs_context` được gõ sau một cụm tiếng Anh ("he is"), còn các dòng `transform` chạy khi tắt xử lý song ngữ (`Sources/VieLotusCLI/main.swift`) |
| VieLotus-IM, từ trùng không có ngữ cảnh | 362 / 369 (Swift), 369 / 369 (C++) | cùng harness đó chạy lại các dòng từ trùng mà không có ngữ cảnh (`README.md`, `docs/TEST_STATUS.md`, `docs/BENCHMARK_REPORT.md`) |
| TrioKey 0.1.0 | 8.361 / 9.084 (92,0%); bỏ qua 9 dòng có dấu cách | mọi dòng được gõ vào một lõi gõ mới, thiết lập mặc định, không có ngữ cảnh (`crates/triokey-engine/tests/vielotus.rs`) |

Hai điều cần lưu ý. Một là số dòng không khớp và chưa tìm ra lý do: file chép về có 9.093 dòng dữ liệu (9.084 chạy + 9 bỏ qua), trong khi tổng mà VieLotus-IM công bố là 9.091. Hai là README của VieLotus-IM nói rõ các con số của họ là kết quả trên bộ test hồi quy, không phải lời khẳng định gõ không bao giờ sai; con số của TrioKey cũng vậy.

Bộ test này thể hiện các lựa chọn của VieLotus-IM, nên khoảng cách một phần là chất lượng, một phần là khác quan điểm, một phần là do thiếu ngữ cảnh. Một số chỗ TrioKey trượt là có chủ ý: nó giữ nguyên `less` và `off` trong khi bộ test mong `les` và `of`, và nó mặc định `has`, `is`, `of` là tiếng Anh trong khi bộ test mong cách đọc tiếng Việt khi không có ngữ cảnh. Số khác là điểm yếu thật: phím dấu gõ lặp trong một từ nằm ngoài danh sách tiếng Anh làm mất một chữ (`passwd` → `paswd`), và nhiều từ có ở cả hai thứ tiếng được quyết định bằng một bảng mặc định chỉnh tay chứ không bằng thống kê. Phân tích đầy đủ ở [decisions/0004-regression-baseline.md](decisions/0004-regression-baseline.md) và [decisions/0005-tie-defaults.md](decisions/0005-tie-defaults.md).

## Nguồn, theo từng dòng

### VKey

- **C++20, GPL-3.0, Windows 10/11, viết lại từ một fork của OpenKey**: `README.md` (phần giới thiệu, yêu cầu, bảng kiến trúc). Phần giới thiệu nói VKey được viết lại từ NextKey, một fork của OpenKey, và lõi gõ có tham khảo mã nguồn UniKey cho các quy tắc âm vị.
- **Ngoại lệ về giấy phép**: `README.md`, phần chính sách bảo mật: từ bản v4.3 có một thư viện nguồn đóng tuỳ chọn viết bằng Rust lo phần sửa chính tả nâng cao; mặc định tắt, và bản Classic phát hành chính thức là 100% mã nguồn mở, không nạp được thư viện này.
- **Hook là đường chính, TSF tuỳ chọn**: bảng kiến trúc trong `README.md` (ứng dụng chứa hook engine; `VKeyTSF` là một DLL đăng ký như bộ gõ của Windows) và `docs/GUIDE.md` mục 11, nơi nói hook vẫn là phương thức nhập chính trừ khi một ứng dụng được thêm vào danh sách TSF.
- **CJK theo ngôn ngữ của layout**: `README.md` (mục "Auto-disable on CJK": tắt gõ tiếng Việt khi layout bàn phím là CJK), `docs/GUIDE.md` mục 5, và mã nguồn: `src/app/system/HookEngine.cpp`, hàm `IsIncompatibleLayout`, so ngôn ngữ chính của layout với Nhật, Trung, Hàn; `src/core/CjkSwitchDecision.h` mô tả máy trạng thái trong phần chú thích đầu file, gồm cả tuỳ chọn của người dùng (`cjkAutoSwitchEnabled`) và hai ngoại lệ (`isExcluded`, `isForcedVietnamese`). Tìm trong mã nguồn hook engine các API đọc trạng thái mở của IME (`ImmGetOpenStatus`, `WM_IME_CONTROL`, `ImmGetDefaultIMEWnd`, compartment mở/đóng bàn phím của TSF) thì không thấy chỗ nào dùng cho quyết định này.
- **Chế độ V/E, phím chuyển, nhớ theo ứng dụng, luật theo tên miền**: `README.md` (Smart Switch per app; "VKey Browser", ghi là thử nghiệm, áp chế độ theo từng tên miền), `docs/GUIDE.md`.
- **Kiểm tra chính tả trả lại phím gốc**: `docs/GUIDE.md` mục 9, "Khôi phục từ với phím sai", mặc định bật: khi một từ không qua được kiểm tra chính tả tiếng Việt và người dùng bấm Space, các phím gốc được trả lại. `README.md` cũng nêu tính năng phát hiện tiếng Anh.
- **Kiểu gõ, bảng mã, gõ tắt**: `README.md`, danh sách tính năng bản tiếng Anh.
- **Chạy quyền Administrator**: `docs/GUIDE.md` (ghi chú về game chạy quyền cao) và `src/app/system/StartupHelper.h` (khởi động lại ở chế độ admin, tự khởi động qua Task Scheduler).
- **Kiểm tra cập nhật, winget**: `README.md`.

### Vikey

- **GPL-3.0, dựa trên OpenKey, các nền tảng**: `README.md`.
- **Lõi gõ và ứng dụng Windows bằng C++**: `Engine/*.cpp`, `Windows/App/*.cpp`; phần build macOS trong `README.md` dùng Xcode.
- **Hook + `SendInput`**: `Windows/App/Core.cpp` (`SetWindowsHookExW(WH_KEYBOARD_LL, …)`, `SendInput`).
- **Nhường theo ngôn ngữ của layout**: `Windows/App/Core.cpp`, hàm `updateOtherInputLanguage`: ngôn ngữ layout của cửa sổ đang được dùng (foreground thread) được so với Anh, Việt và trung tính; nếu khác thì hook cho phím đi thẳng, khi tuỳ chọn `vOtherLanguage` đang bật. `README.md` nói Vikey tự nhường cho các ngôn ngữ nhập khác (Nhật, Trung…). Tìm trong `Windows/` và `Engine/` các API đọc trạng thái mở của IME thì không thấy.
- **Tự chuyển Anh/Việt, để nguyên từ lập trình**: danh sách tính năng trong `README.md`; `DECISIONS.md` (ví dụ mục 22, 30, 34 về từ trùng và cặp từ).
- **Hoàn tác bằng Backspace**: `DECISIONS.md` mục 61: sau khi một từ được sửa lúc gõ dấu cách, Backspace đầu chỉ xoá dấu cách, Backspace thứ hai mới hoàn tác; mục này ghi rõ cái giá là hai lần bấm thay vì một.
- **Telex, Simple Telex; không có VNI, bảng mã hay gõ tắt**: `Windows/App/Settings.cpp` chỉ cho chọn Telex, Telex đơn giản 1 và Telex đơn giản 2; một chú thích trong `Windows/App/Prefs.cpp` nói VNI không có trong Vikey; `DECISIONS.md` mục 56 (Simple Telex) và danh sách tính năng đã bỏ (VNI, các bảng mã ngoài Unicode, gõ tắt).
- **Không tìm thấy chế độ admin**: `Windows/App/Vikey.manifest` yêu cầu `asInvoker`; tìm trong `Windows/App/*.cpp` một đường nâng quyền thì không thấy. (`Windows/PLAN.md`, một ghi chú kế hoạch, có nhắc admin mode dưới dạng đề xuất.)
- **Nhớ theo ứng dụng và theo trang web**: `README.md`.
- **Tự cập nhật, báo cáo lỗi có hỏi trước**: `Windows/App/Updater.cpp`, `Windows/App/CrashReporter.cpp`. `README.md` có mô tả một nhật ký gõ phải tự bật để tinh chỉnh, mặc định tắt.
- **Bộ cài, phiên bản**: `Windows/installer/Vikey.iss`; `DECISIONS.md` mục 61 ("Shipped in 1.0.33").

### VieLotus-IM

- **MIT, nền tảng, ngôn ngữ lập trình, nơi tải**: `README.md` (bảng nền tảng: bản phát hành, Microsoft Store, gói Debian), `docs/ARCHITECTURE.md`.
- **Bản Windows là một text service TSF**: `README.md` ("C++20 engine and TSF shell"), `docs/ARCHITECTURE.md`, và `CrossPlatform/windows-tsf/README.md`, nơi mô tả một text input processor dạng COM chạy trong tiến trình, đăng ký dưới `vi-VN` và xuất hiện trong danh sách bàn phím của Windows sau khi đăng ký.
- **Telex và VNI, gõ tắt**: `CrossPlatform/windows-tsf/src/PrefsStore.h` (`isVni`, `macroEnabled`); `PRIVACY.md` có nhắc tới macro.
- **Bộ test hồi quy và kết quả của chính dự án**: `Sources/VieLotusCLI/Resources/test_suite.csv`; `README.md` và `docs/TEST_STATUS.md` ghi 9.091/9.091 trên bộ chính và 362/369 (Swift) hoặc 369/369 (C++) khi chạy lại các từ trùng không có ngữ cảnh; `Sources/VieLotusCLI/main.swift` cho thấy từng loại dòng được chạy thế nào.
- **Mạng**: `README.md` nói việc gõ được xử lý cục bộ và phần cập nhật trên macOS có liên lạc với GitHub; bản Windows thì chưa kiểm.

### TrioKey

- Mọi thứ ở cột TrioKey lấy từ repo này: [README](../README.vi.md), [docs/decisions/](decisions/) và mã nguồn trong `crates/`.
