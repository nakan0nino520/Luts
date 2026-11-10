//! Lõi tính toán của LUT Studio. Không phụ thuộc trình duyệt, kiểm thử bằng `cargo test`.

use std::collections::HashMap;

/* ---------- .cube ---------- */

pub struct Cube {
    pub n: usize,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub title: String,
    pub data: Vec<f32>,
}

fn triple(p: &[&str]) -> Option<[f32; 3]> {
    if p.len() < 3 {
        return None;
    }
    Some([
        p[0].parse::<f32>().ok()?,
        p[1].parse::<f32>().ok()?,
        p[2].parse::<f32>().ok()?,
    ])
}

/// Đọc LUT 3D dạng .cube. Lỗi trả về "1D" (chưa hỗ trợ LUT 1D) hoặc "bad" (tệp sai).
pub fn parse_cube(text: &str) -> Result<Cube, String> {
    let mut n = 0usize;
    let mut min = [0.0f32; 3];
    let mut max = [1.0f32; 3];
    let mut title = String::new();
    let mut data: Vec<f32> = Vec::new();
    for line in text.lines() {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let p: Vec<&str> = s.split_whitespace().collect();
        match p[0] {
            "TITLE" => title = s["TITLE".len()..].trim().replace('"', ""),
            "LUT_3D_SIZE" => n = p.get(1).and_then(|x| x.parse::<usize>().ok()).unwrap_or(0),
            "LUT_1D_SIZE" => return Err("1D".to_string()),
            "DOMAIN_MIN" => {
                if let Some(t) = triple(&p[1..]) {
                    min = t;
                }
            }
            "DOMAIN_MAX" => {
                if let Some(t) = triple(&p[1..]) {
                    max = t;
                }
            }
            _ => {
                let c = p[0].chars().next().unwrap_or(' ');
                if (c == '-' || c == '.' || c.is_ascii_digit()) && p.len() >= 3 {
                    match triple(&p) {
                        Some(t) => data.extend_from_slice(&t),
                        None => return Err("bad".to_string()),
                    }
                }
            }
        }
    }
    let total = n
        .checked_mul(n)
        .and_then(|x| x.checked_mul(n))
        .and_then(|x| x.checked_mul(3));
    if n == 0 || n > 256 || total != Some(data.len()) {
        return Err("bad".to_string());
    }
    Ok(Cube { n, min, max, title, data })
}

/* ---------- WAV ---------- */

fn to_i16(s: f32) -> i16 {
    let v = s.clamp(-1.0, 1.0);
    if v < 0.0 {
        (v * 32768.0) as i16
    } else {
        (v * 32767.0) as i16
    }
}

/// WAV PCM 16-bit stereo.
pub fn wav_encode(left: &[f32], right: &[f32], sr: u32) -> Vec<u8> {
    let n = left.len().min(right.len());
    let data_len = (n * 4) as u32;
    let mut o = Vec::with_capacity(44 + n * 4);
    o.extend_from_slice(b"RIFF");
    o.extend_from_slice(&(36 + data_len).to_le_bytes());
    o.extend_from_slice(b"WAVEfmt ");
    o.extend_from_slice(&16u32.to_le_bytes());
    o.extend_from_slice(&1u16.to_le_bytes());
    o.extend_from_slice(&2u16.to_le_bytes());
    o.extend_from_slice(&sr.to_le_bytes());
    o.extend_from_slice(&(sr * 4).to_le_bytes());
    o.extend_from_slice(&4u16.to_le_bytes());
    o.extend_from_slice(&16u16.to_le_bytes());
    o.extend_from_slice(b"data");
    o.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..n {
        for s in [left[i], right[i]] {
            o.extend_from_slice(&to_i16(s).to_le_bytes());
        }
    }
    o
}

/* ---------- GIF ---------- */

fn put(out: &mut Vec<u8>, bits: &mut u32, nb: &mut u32, c: u32, z: u32) {
    *bits |= c << *nb;
    *nb += z;
    while *nb >= 8 {
        out.push((*bits & 255) as u8);
        *bits >>= 8;
        *nb -= 8;
    }
}

/// LZW cho GIF (giữ nguyên hành vi bản JavaScript cũ).
fn lzw(px: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if px.is_empty() {
        return out;
    }
    let (mut bits, mut nb) = (0u32, 0u32);
    let mut m: HashMap<u32, u32> = HashMap::new();
    let (mut nx, mut z) = (258u32, 9u32);
    let mut pre = px[0] as u32;
    put(&mut out, &mut bits, &mut nb, 256, z);
    for &c in &px[1..] {
        let c = c as u32;
        let k = (pre << 8) | c;
        if let Some(&g) = m.get(&k) {
            pre = g;
        } else {
            put(&mut out, &mut bits, &mut nb, pre, z);
            if nx > (1 << z) - 1 && z < 12 {
                z += 1;
            }
            if nx < 4096 {
                m.insert(k, nx);
                nx += 1;
            } else {
                put(&mut out, &mut bits, &mut nb, 256, z);
                m.clear();
                nx = 258;
                z = 9;
            }
            pre = c;
        }
    }
    put(&mut out, &mut bits, &mut nb, pre, z);
    put(&mut out, &mut bits, &mut nb, 257, z);
    if nb > 0 {
        out.push((bits & 255) as u8);
    }
    out
}

/// `frames` là `count` khung liền nhau, mỗi khung w*h byte chỉ số bảng màu 3-3-2.
pub fn gif_encode(frames: &[u8], w: u16, h: u16, count: usize) -> Vec<u8> {
    let px = w as usize * h as usize;
    let mut o = Vec::new();
    o.extend_from_slice(b"GIF89a");
    o.extend_from_slice(&w.to_le_bytes());
    o.extend_from_slice(&h.to_le_bytes());
    o.extend_from_slice(&[247, 0, 0]);
    for i in 0..256u32 {
        o.push((((i >> 5) * 255) / 7) as u8);
        o.push(((((i >> 2) & 7) * 255) / 7) as u8);
        o.push(((i & 3) * 85) as u8);
    }
    o.extend_from_slice(&[33, 255, 11]);
    o.extend_from_slice(b"NETSCAPE2.0");
    o.extend_from_slice(&[3, 1, 0, 0, 0]);
    for f in 0..count {
        if (f + 1) * px > frames.len() {
            break;
        }
        let p = &frames[f * px..(f + 1) * px];
        o.extend_from_slice(&[33, 249, 4, 0]);
        o.extend_from_slice(&10u16.to_le_bytes());
        o.extend_from_slice(&[0, 0, 44]);
        o.extend_from_slice(&0u16.to_le_bytes());
        o.extend_from_slice(&0u16.to_le_bytes());
        o.extend_from_slice(&w.to_le_bytes());
        o.extend_from_slice(&h.to_le_bytes());
        o.extend_from_slice(&[0, 8]);
        for c in lzw(p).chunks(255) {
            o.push(c.len() as u8);
            o.extend_from_slice(c);
        }
        o.push(0);
    }
    o.push(59);
    o
}

/* ---------- MP4 / M4A ---------- */

const TS: u32 = 90000;

pub struct VideoTrack<'a> {
    pub hevc: bool,
    pub desc: &'a [u8],
    pub w: u32,
    pub h: u32,
    pub fps: f64,
    pub data: &'a [u8],
    pub sizes: &'a [u32],
    pub keys: &'a [u8],
}

pub struct AudioTrack<'a> {
    pub sr: u32,
    pub asc: &'a [u8],
    pub data: &'a [u8],
    pub sizes: &'a [u32],
}

struct Smp {
    size: usize,
    key: bool,
    dl: u32,
    off: usize,
}

fn bx(tag: &str, body: &[u8]) -> Vec<u8> {
    let mut o = Vec::with_capacity(body.len() + 8);
    o.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
    o.extend_from_slice(tag.as_bytes());
    o.extend_from_slice(body);
    o
}

struct W(Vec<u8>);
impl W {
    fn new() -> W {
        W(Vec::new())
    }
    fn b(mut self, x: &[u8]) -> W {
        self.0.extend_from_slice(x);
        self
    }
    fn u16(self, n: u16) -> W {
        self.b(&n.to_be_bytes())
    }
    fn u32(self, n: u32) -> W {
        self.b(&n.to_be_bytes())
    }
    fn z(self, n: usize) -> W {
        self.b(&vec![0u8; n])
    }
    fn s(self, t: &str) -> W {
        self.b(t.as_bytes())
    }
}

fn mtx() -> Vec<u8> {
    let mut w = W::new();
    for n in [0x10000u32, 0, 0, 0, 0x10000, 0, 0, 0, 0x40000000] {
        w = w.u32(n);
    }
    w.0
}

fn video_samples(v: &VideoTrack) -> Vec<Smp> {
    let fps = v.fps;
    v.sizes
        .iter()
        .enumerate()
        .map(|(i, &sz)| {
            let a = ((i as f64 + 1.0) * TS as f64 / fps).round() as u32;
            let b = (i as f64 * TS as f64 / fps).round() as u32;
            Smp { size: sz as usize, key: v.keys.get(i).copied().unwrap_or(0) != 0, dl: a - b, off: 0 }
        })
        .collect()
}

fn audio_samples(a: &AudioTrack) -> Vec<Smp> {
    a.sizes.iter().map(|&sz| Smp { size: sz as usize, key: true, dl: 1024, off: 0 }).collect()
}

fn trak(id: u32, vid: bool, ts: u32, m: &[Smp], entry: &[u8], w: u32, h: u32) -> (u32, Vec<u8>) {
    let dur: u32 = m.iter().map(|x| x.dl).sum();
    let mdur = (dur as f64 / ts as f64 * 1000.0).round() as u32;
    let mut runs: Vec<(u32, u32)> = Vec::new();
    for x in m {
        if let Some(l) = runs.last_mut() {
            if l.1 == x.dl {
                l.0 += 1;
                continue;
            }
        }
        runs.push((1, x.dl));
    }
    let stsd = bx("stsd", &W::new().z(4).u32(1).b(entry).0);
    let mut sb = W::new().z(4).u32(runs.len() as u32);
    for r in &runs {
        sb = sb.u32(r.0).u32(r.1);
    }
    let stts = bx("stts", &sb.0);
    let mut parts: Vec<Vec<u8>> = vec![stsd, stts];
    if vid {
        let ss: Vec<u32> = m.iter().enumerate().filter(|(_, x)| x.key).map(|(i, _)| i as u32 + 1).collect();
        let mut b = W::new().z(4).u32(ss.len() as u32);
        for s in ss {
            b = b.u32(s);
        }
        parts.push(bx("stss", &b.0));
    }
    parts.push(bx("stsc", &W::new().z(4).u32(1).u32(1).u32(1).u32(1).0));
    let mut b = W::new().z(4).u32(0).u32(m.len() as u32);
    for x in m {
        b = b.u32(x.size as u32);
    }
    parts.push(bx("stsz", &b.0));
    let mut b = W::new().z(4).u32(m.len() as u32);
    for x in m {
        b = b.u32(x.off as u32);
    }
    parts.push(bx("stco", &b.0));
    let stbl = bx("stbl", &parts.concat());
    let head = if vid {
        bx("vmhd", &W::new().b(&[0, 0, 0, 1]).z(8).0)
    } else {
        bx("smhd", &W::new().z(8).0)
    };
    let dinf = bx("dinf", &bx("dref", &W::new().z(4).u32(1).b(&bx("url ", &[0, 0, 0, 1])).0));
    let minf = bx("minf", &[head, dinf, stbl].concat());
    let tkhd = bx(
        "tkhd",
        &W::new()
            .b(&[0, 0, 0, 7]).u32(0).u32(0).u32(id).z(4).u32(mdur).z(8)
            .u16(0).u16(0).u16(if vid { 0 } else { 0x100 }).u16(0)
            .b(&mtx()).u32(w << 16).u32(h << 16).0,
    );
    let mdhd = bx("mdhd", &W::new().z(4).u32(0).u32(0).u32(ts).u32(dur).u16(0x55c4).u16(0).0);
    let hdlr = bx(
        "hdlr",
        &W::new().z(4).z(4).s(if vid { "vide" } else { "soun" }).z(12).s(if vid { "V" } else { "S" }).b(&[0]).0,
    );
    let mdia = bx("mdia", &[mdhd, hdlr, minf].concat());
    (mdur, bx("trak", &[tkhd, mdia].concat()))
}

fn moov(v: Option<&VideoTrack>, a: Option<&AudioTrack>, vs: &mut [Smp], aus: &mut [Smp], base: usize) -> Vec<u8> {
    let mut o = base;
    for x in vs.iter_mut().chain(aus.iter_mut()) {
        x.off = o;
        o += x.size;
    }
    let mut traks: Vec<u8> = Vec::new();
    let (mut id, mut md) = (1u32, 0u32);
    if let Some(v) = v {
        let (tag, cfg) = if v.hevc { ("hvc1", "hvcC") } else { ("avc1", "avcC") };
        let entry = bx(
            tag,
            &W::new()
                .z(6).u16(1).z(16).u16(v.w as u16).u16(v.h as u16)
                .u32(0x480000).u32(0x480000).z(4).u16(1).z(32).u16(0x18).u16(0xffff)
                .b(&bx(cfg, v.desc)).0,
        );
        let (d, t) = trak(id, true, TS, vs, &entry, v.w, v.h);
        id += 1;
        md = md.max(d);
        traks.extend(t);
    }
    if let Some(a) = a {
        let a0 = *a.asc.get(0).unwrap_or(&0);
        let a1 = *a.asc.get(1).unwrap_or(&0);
        let esds = bx(
            "esds",
            &W::new().z(4).b(&[3, 25, 0, 0, 0]).b(&[4, 17, 0x40, 0x15, 0, 0, 0])
                .u32(128000).u32(128000).b(&[5, 2, a0, a1]).b(&[6, 1, 2]).0,
        );
        let entry = bx(
            "mp4a",
            &W::new().z(6).u16(1).z(8).u16(2).u16(16).z(4).u32(a.sr.wrapping_mul(65536)).b(&esds).0,
        );
        let (d, t) = trak(id, false, a.sr, aus, &entry, 0, 0);
        id += 1;
        md = md.max(d);
        traks.extend(t);
    }
    let mvhd = bx(
        "mvhd",
        &W::new().z(4).u32(0).u32(0).u32(1000).u32(md).u32(0x10000).u16(0x100).z(10)
            .b(&mtx()).z(24).u32(id).0,
    );
    bx("moov", &[mvhd, traks].concat())
}

/// Ghép các mẫu H.264/HEVC và/hoặc AAC thành MP4 (hoặc M4A nếu chỉ có âm thanh).
pub fn mux_mp4(v: Option<&VideoTrack>, a: Option<&AudioTrack>) -> Vec<u8> {
    let mut vs = v.map(video_samples).unwrap_or_default();
    let mut aus = a.map(audio_samples).unwrap_or_default();
    let ft = if v.is_some() {
        bx("ftyp", &W::new().s("isom").u32(512).s("isom").s("iso2").s("avc1").s("mp41").0)
    } else {
        bx("ftyp", &W::new().s("M4A ").u32(0).s("M4A ").s("mp42").s("isom").0)
    };
    let mv0 = moov(v, a, &mut vs, &mut aus, 0);
    let base = ft.len() + mv0.len() + 8;
    let mv = moov(v, a, &mut vs, &mut aus, base);
    let total: usize = vs.iter().chain(aus.iter()).map(|x| x.size).sum();
    let mut out = ft;
    out.extend_from_slice(&mv);
    out.extend_from_slice(&((total + 8) as u32).to_be_bytes());
    out.extend_from_slice(b"mdat");
    if let Some(v) = v {
        out.extend_from_slice(v.data);
    }
    if let Some(a) = a {
        out.extend_from_slice(a.data);
    }
    out
}

/* ---------- phụ đề ---------- */

/// SRT -> WebVTT (đổi dấu phẩy thành dấu chấm ở dòng thời gian). VTT giữ nguyên.
pub fn srt_to_vtt(text: &str) -> String {
    if text.starts_with("WEBVTT") {
        return text.to_string();
    }
    let mut out = String::from("WEBVTT\n\n");
    for line in text.replace('\r', "").lines() {
        if line.contains("-->") {
            out.push_str(&line.replace(',', "."));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUBE: &str = "TITLE \"Test\"\nLUT_3D_SIZE 2\nDOMAIN_MIN 0 0 0\nDOMAIN_MAX 1 1 1\n\
0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n";

    fn top(b: &[u8]) -> Vec<(String, usize)> {
        let mut i = 0;
        let mut v = Vec::new();
        while i + 8 <= b.len() {
            let sz = u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize;
            v.push((String::from_utf8_lossy(&b[i + 4..i + 8]).to_string(), sz));
            if sz < 8 {
                break;
            }
            i += sz;
        }
        v
    }

    #[test]
    fn doc_cube_hop_le() {
        let c = parse_cube(CUBE).unwrap();
        assert_eq!(c.n, 2);
        assert_eq!(c.title, "Test");
        assert_eq!(c.data.len(), 24);
        assert_eq!(c.max, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn doc_cube_loi() {
        assert_eq!(parse_cube("LUT_1D_SIZE 4\n").err().unwrap(), "1D");
        assert_eq!(parse_cube("LUT_3D_SIZE 2\n0 0 0\n").err().unwrap(), "bad");
        assert_eq!(parse_cube("").err().unwrap(), "bad");
    }

    #[test]
    fn wav_dung_dinh_dang() {
        let w = wav_encode(&[0.0, 1.0, -1.0], &[0.0, 1.0, -1.0], 48000);
        assert_eq!(w.len(), 44 + 12);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes([w[4], w[5], w[6], w[7]]), 48);
        assert_eq!(i16::from_le_bytes([w[48], w[49]]), 32767);
    }

    #[test]
    fn gif_co_dau_va_kich_thuoc() {
        let frames = [0u8, 1, 2, 3, 0, 1, 2, 3];
        let g = gif_encode(&frames, 2, 2, 2);
        assert_eq!(&g[0..6], b"GIF89a");
        assert_eq!(&g[6..8], &[2, 0]);
        assert_eq!(*g.last().unwrap(), 59);
    }

    #[test]
    fn mp4_cau_truc_hop_le() {
        let data = [7u8; 30];
        let v = VideoTrack {
            hevc: false, desc: &[1, 2, 3], w: 64, h: 48, fps: 30.0,
            data: &data, sizes: &[10, 10, 10], keys: &[1, 0, 0],
        };
        let out = mux_mp4(Some(&v), None);
        let b = top(&out);
        let names: Vec<&str> = b.iter().map(|x| x.0.as_str()).collect();
        assert_eq!(names, ["ftyp", "moov", "mdat"]);
        assert_eq!(b.iter().map(|x| x.1).sum::<usize>(), out.len());
        assert_eq!(b[2].1, 38);
    }

    #[test]
    fn m4a_chi_co_am_thanh() {
        let data = [1u8; 8];
        let a = AudioTrack { sr: 48000, asc: &[0x11, 0x90], data: &data, sizes: &[4, 4] };
        let out = mux_mp4(None, Some(&a));
        assert_eq!(&out[4..8], b"ftyp");
        assert_eq!(&out[8..12], b"M4A ");
    }

    #[test]
    fn srt_sang_vtt() {
        let s = srt_to_vtt("1\r\n00:00:01,500 --> 00:00:02,000\r\nXin chào, bạn\r\n");
        assert!(s.starts_with("WEBVTT\n\n"));
        assert!(s.contains("00:00:01.500 --> 00:00:02.000"));
        assert!(s.contains("Xin chào, bạn"));
        assert_eq!(srt_to_vtt("WEBVTT\n\nx"), "WEBVTT\n\nx");
    }
}
