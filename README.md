# LUT Studio (lõi Rust)

Xem trước màu điện ảnh bằng LUT `.cube` cho ảnh và video, chỉnh âm, xuất MP4, GIF, WAV. Giao diện chạy trong trình duyệt; phần xử lý dữ liệu nặng viết bằng **Rust** và biên dịch ra **WebAssembly**.

## Phần nào viết bằng Rust

| Chức năng                             | Hàm Rust     |
| ------------------------------------- | ------------ |
| Đọc và kiểm tra tệp LUT `.cube`       | `parse_cube` |
| Ghi tệp MP4 (H.264/HEVC + AAC) và M4A | `mux_mp4`    |
| Mã hóa GIF (LZW)                      | `gif_encode` |
| Mã hóa WAV 16-bit                     | `wav_encode` |
| Đổi phụ đề SRT sang VTT               | `srt_to_vtt` |

Giao diện, WebGL (áp LUT), WebCodecs và Web Audio vẫn là JavaScript, vì đó là API riêng của trình duyệt. Nếu WASM không tải được, trang tự dùng bản JavaScript cũ nên vẫn chạy.

## Cấu trúc

```
src/engine.rs                    lõi tính toán, có test
src/lib.rs                       lớp nối wasm-bindgen
web/index.html                   giao diện LUT Studio
web/rust-bridge.js               thay các hàm JS cũ bằng hàm Rust
web/perf-patch.js                giảm giật khi phát video
.github/workflows/pages.yml      tự build Rust ra WebAssembly và đăng lên GitHub Pages
```

## Đưa lên GitHub Pages (làm được trên điện thoại)

1. Đưa các tệp lên repo (thư mục `src`, `web`, `Cargo.toml`, `README.md`, `LICENSE`).
2. **Settings → Pages → Build and deployment → Source: GitHub Actions.**
3. Thư mục `.github` thường không lên được khi tải tệp từ điện thoại. Tạo bằng tay: tab **Actions → set up a workflow yourself**, xóa hết nội dung trong ô soạn thảo, dán nội dung tệp `pages.yml`, rồi **Commit changes**.
4. Đợi tab **Actions** chạy xong (lần đầu khoảng 6–8 phút). Xanh là xong.
5. Mở trang, thêm `?rust` vào cuối địa chỉ để thấy lõi Rust có đang bật không.

Nếu bước biên dịch Rust lỗi, trang vẫn được đăng và chạy bằng bản JavaScript dự phòng; Actions hiện cảnh báo màu cam kèm dòng lỗi.

## Chạy trên máy

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

cargo test
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir web/pkg --out-name lut_studio target/wasm32-unknown-unknown/release/lut_studio.wasm
python3 -m http.server -d web 8080   # mở http://localhost:8080
```

`wasm-bindgen-cli` phải cùng phiên bản với `wasm-bindgen` trong `Cargo.lock`.

## Trang chỉ hiện README?

Pages đang ở chế độ "Deploy from a branch". Vào **Settings → Pages**, đổi Source thành **GitHub Actions**, rồi chạy lại workflow ở tab Actions (nút **Run workflow**).

## Giấy phép

MIT
