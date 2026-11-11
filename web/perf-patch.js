// Giảm giật khi phát video có LUT.
// Thêm vào index.html, ngay trước </body> và SAU thẻ script chính:
//   <script src="perf-patch.js"></script>
// Mỗi mục được bọc riêng: nếu index.html đổi tên một hàm, chỉ mục đó bị bỏ qua và trang vẫn chạy.
(function () {
  const step = (ten, fn) => {
    try { fn(); } catch (e) { console.info('perf-patch: bỏ qua mục "' + ten + '"', e); }
  };

  // 1) Kính mờ đặt đè lên video buộc máy làm mờ lại mỗi khung hình. Tắt trong khung xem.
  step('kính mờ trong khung xem', () => {
    const st = document.createElement('style');
    st.textContent = '#stage .glass{-webkit-backdrop-filter:none!important;backdrop-filter:none!important;background:rgba(20,20,26,.62)!important}';
    document.head.appendChild(st);
  });

  // 2) Chỉ vẽ lại khi có khung hình video mới (trước đây vẽ theo từng nhịp màn hình, kể cả khi chưa có khung mới).
  step('vẽ theo khung video mới', () => {
    const hasRVFC = 'requestVideoFrameCallback' in V;
    let fresh = true;
    if (hasRVFC) {
      const f = () => { fresh = true; V.requestVideoFrameCallback(f); };
      V.requestVideoFrameCallback(f);
    }
    const od = draw;
    draw = function () {
      if (hasRVFC && kind === 'v' && src === V && !V.paused && !fresh) return;
      fresh = false;
      od();
    };
  });

  // 3) Nền video mờ phải đọc ngược từ canvas WebGL rất tốn. Giảm còn 2 lần mỗi giây.
  step('nền video mờ', () => {
    const ov = vbTick;
    let last = 0;
    vbTick = function (t) {
      if (t - last < 500) return;
      last = t;
      ov(t);
    };
  });

  // 4) Khung xem trước của video tối đa 1280 px ở cạnh dài (xuất video vẫn dùng độ phân giải đã chọn).
  step('khung xem tối đa 1280 px', () => {
    const orr = ready;
    ready = function () {
      orr();
      if (kind !== 'v') return;
      const k = Math.min(1, 1280 / Math.max(cv.width, cv.height));
      if (k < 1) {
        cv.width = Math.round(cv.width * k);
        cv.height = Math.round(cv.height * k);
        vbInit();
      }
      dirty = true;
    };
  });
})();
