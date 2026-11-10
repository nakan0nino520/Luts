//! Lớp nối `engine` với JavaScript qua wasm-bindgen.
pub mod engine;

use wasm_bindgen::prelude::*;

/// Kết quả đọc tệp .cube.
#[wasm_bindgen]
pub struct Cube {
    inner: engine::Cube,
}

#[wasm_bindgen]
impl Cube {
    #[wasm_bindgen(getter)]
    pub fn n(&self) -> u32 {
        self.inner.n as u32
    }
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.inner.title.clone()
    }
    pub fn dmin(&self) -> Vec<f32> {
        self.inner.min.to_vec()
    }
    pub fn dmax(&self) -> Vec<f32> {
        self.inner.max.to_vec()
    }
    pub fn data(&self) -> Vec<f32> {
        self.inner.data.clone()
    }
}

/// Lỗi trả về là chuỗi "1D" hoặc "bad", giống bản JavaScript.
#[wasm_bindgen]
pub fn parse_cube(text: &str) -> Result<Cube, JsValue> {
    engine::parse_cube(text)
        .map(|inner| Cube { inner })
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn wav_encode(left: &[f32], right: &[f32], sample_rate: u32) -> Vec<u8> {
    engine::wav_encode(left, right, sample_rate)
}

#[wasm_bindgen]
pub fn gif_encode(frames: &[u8], w: u16, h: u16, count: u32) -> Vec<u8> {
    engine::gif_encode(frames, w, h, count as usize)
}

/// Video để trống (`vsizes` rỗng) thì ra M4A; âm thanh để trống thì ra MP4 không tiếng.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn mux_mp4(
    hevc: bool,
    desc: &[u8],
    w: u32,
    h: u32,
    fps: f64,
    vdata: &[u8],
    vsizes: &[u32],
    vkeys: &[u8],
    sr: u32,
    asc: &[u8],
    adata: &[u8],
    asizes: &[u32],
) -> Vec<u8> {
    let v = engine::VideoTrack { hevc, desc, w, h, fps, data: vdata, sizes: vsizes, keys: vkeys };
    let a = engine::AudioTrack { sr, asc, data: adata, sizes: asizes };
    engine::mux_mp4(
        if vsizes.is_empty() { None } else { Some(&v) },
        if asizes.is_empty() { None } else { Some(&a) },
    )
}

#[wasm_bindgen]
pub fn srt_to_vtt(text: &str) -> String {
    engine::srt_to_vtt(text)
}
