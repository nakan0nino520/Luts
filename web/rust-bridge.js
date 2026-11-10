// Nối lõi Rust vào LUT Studio. Thêm một dòng vào cuối index.html, ngay trước </body>:
//   <script type="module" src="rust-bridge.js"></script>
// Nếu WASM không tải được (ví dụ mở file trực tiếp), trang tự dùng bản JavaScript cũ.
// Mở trang với ?rust ở cuối địa chỉ (ví dụ .../Luts-Rust/?rust) để thấy trên màn hình lõi Rust có đang bật không.
const badge = (ok, e) => {
  if (!/[?&]rust\b/.test(location.search)) return;
  const d = document.createElement('div');
  d.textContent = ok ? 'Lõi Rust/WASM: đang bật' : 'Đang dùng JavaScript dự phòng: ' + ((e && e.message) || e);
  d.style.cssText = 'position:fixed;left:8px;bottom:8px;z-index:99999;max-width:90vw;padding:6px 10px;border-radius:10px;font:12px system-ui,sans-serif;color:#fff;background:' + (ok ? '#1b7f3b' : '#8a4b00');
  document.body.appendChild(d);
};
try {
  const R = await import('./pkg/lut_studio.js');
  await R.default();
  window.R = R;

  // Đọc .cube: trả về đúng hình dạng cũ {N, mn, mx, title, d}
  window.parseCube = t => {
    const c = R.parse_cube(t);
    const o = { N: c.n, mn: Array.from(c.dmin()), mx: Array.from(c.dmax()), title: c.title, d: c.data() };
    c.free();
    return o;
  };

  // WAV 16-bit stereo
  window.wav = b => new Blob([R.wav_encode(
    b.getChannelData(0), b.getChannelData(b.numberOfChannels > 1 ? 1 : 0), b.sampleRate)], { type: 'audio/wav' });

  // GIF: gom các khung thành một mảng liền rồi mã hóa
  window.gifEnc = (fr, W, H) => {
    const a = new Uint8Array(fr.length * W * H);
    fr.forEach((p, i) => a.set(p, i * W * H));
    return new Blob([R.gif_encode(a, W, H, fr.length)], { type: 'image/gif' });
  };

  // MP4 / M4A
  window.mux = (v, a) => {
    const cat = ss => {
      const o = new Uint8Array(ss.reduce((q, x) => q + x.length, 0));
      let p = 0;
      for (const x of ss) { o.set(x, p); p += x.length; }
      return o;
    };
    const vs = v ? v.s : [], as = a ? a.s : [];
    const out = R.mux_mp4(
      !!(v && v.hevc), v ? v.desc : new Uint8Array(0), v ? v.w : 0, v ? v.h : 0, v ? v.fps : 30,
      cat(vs.map(x => x.d)), Uint32Array.from(vs, x => x.d.length), Uint8Array.from(vs, x => x.key ? 1 : 0),
      a ? a.sr : 48000, a ? new Uint8Array(a.asc) : new Uint8Array(2),
      cat(as), Uint32Array.from(as, x => x.length));
    return new Blob([out], { type: v ? 'video/mp4' : 'audio/mp4' });
  };

  // Phụ đề SRT -> VTT
  window.setSub = (raw, ext) => {
    const t = R.srt_to_vtt(raw);
    [...V.querySelectorAll('track')].forEach(x => x.remove());
    const tk = document.createElement('track');
    tk.kind = 'subtitles';
    tk.src = URL.createObjectURL(new Blob([t], { type: 'text/vtt' }));
    V.appendChild(tk);
    tk.track.mode = 'hidden';
    tk.track.oncuechange = () => {
      $('#sub').textContent = [...tk.track.activeCues || []].map(c => c.text.replace(/<[^>]+>/g, '')).join('\n');
    };
  };
  console.info('LUT Studio: lõi Rust/WebAssembly đã bật');
  badge(true);
} catch (e) {
  console.info('LUT Studio: dùng bản JavaScript dự phòng', e);
  badge(false, e);
}
