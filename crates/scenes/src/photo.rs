//! Real photographs in the demo library.
//!
//! Eight public-domain pictures (Ansel Adams's National Archives negatives and NASA's Apollo
//! photographs) are embedded as small JPEGs. They are decoded once, on first use, and resampled to
//! whatever size a scene is asked for, in linear light, then converted to linear Rec.2020 like the
//! procedural scenes. Their sources and licences are listed in `assets/ATTRIBUTION.md`.

use std::sync::{Arc, OnceLock};

use luz_color::{REC2020, SRGB};
use luz_raster::Rgb32f;
use rayon::prelude::*;

/// The embedded photographs; a photo scene's seed indexes this list.
const FILES: [&[u8]; 8] = [
    include_bytes!("../assets/adams-tetons.jpg"),
    include_bytes!("../assets/adams-mountains.jpg"),
    include_bytes!("../assets/adams-orchard.jpg"),
    include_bytes!("../assets/adams-cave.jpg"),
    include_bytes!("../assets/adams-canyon.jpg"),
    include_bytes!("../assets/adams-steam.jpg"),
    include_bytes!("../assets/nasa-earthrise.jpg"),
    include_bytes!("../assets/nasa-blue-marble.jpg"),
];

/// A decoded 8-bit RGB picture (a grey one has equal channels), sRGB-encoded.
pub(crate) struct Decoded {
    pub width: usize,
    pub height: usize,
    data: Vec<u8>,
}

/// How many photographs are embedded (tests check the library lists them all).
#[cfg(test)]
pub(crate) const COUNT: usize = FILES.len();

static CACHE: [OnceLock<Option<Arc<Decoded>>>; FILES.len()] = [const { OnceLock::new() }; FILES.len()];

/// zune-jpeg, not jpeg-decoder: that one fans out onto rayon's global pool, which panics where its
/// threads can't start (the demo library must still load there, see `luz_engine::demo`).
fn decode(bytes: &[u8]) -> Option<Decoded> {
    use zune_core::bytestream::ZCursor;
    use zune_core::colorspace::ColorSpace;
    use zune_core::options::DecoderOptions;
    let opts = DecoderOptions::default().set_max_width(1 << 13).set_max_height(1 << 13).jpeg_set_out_colorspace(ColorSpace::RGB);
    let mut d = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(bytes), opts);
    d.decode_headers().ok()?;
    let info = d.info()?;
    let (width, height) = (usize::from(info.width), usize::from(info.height));
    let data = d.decode().ok()?;
    (width > 0 && height > 0 && data.len() == width.checked_mul(height)?.checked_mul(3)?).then_some(Decoded { width, height, data })
}

/// Photograph `index` decoded (once), or `None` for an unknown index or a damaged embedded file.
pub(crate) fn get(index: u32) -> Option<Arc<Decoded>> {
    let slot = CACHE.get(usize::try_from(index).ok()?)?;
    slot.get_or_init(|| decode(FILES.get(usize::try_from(index).ok()?)?).map(Arc::new)).clone()
}

/// The native size of photograph `index`.
pub fn size(index: u32) -> Option<(u32, u32)> {
    let p = get(index)?;
    Some((u32::try_from(p.width).ok()?, u32::try_from(p.height).ok()?))
}

fn lut() -> &'static [f32; 256] {
    static LUT: OnceLock<[f32; 256]> = OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0.0f32; 256];
        for (i, v) in t.iter_mut().enumerate() {
            *v = luz_color::transfer::srgb_to_linear(i as f32 / 255.0);
        }
        t
    })
}

impl Decoded {
    /// Linear sRGB of the source pixel (clamped to the picture).
    fn linear(&self, x: usize, y: usize, t: &[f32; 256]) -> [f32; 3] {
        let (x, y) = (x.min(self.width - 1), y.min(self.height - 1));
        let i = (y * self.width + x) * 3;
        match self.data.get(i..i + 3) {
            Some([r, g, b]) => [t[usize::from(*r)], t[usize::from(*g)], t[usize::from(*b)]],
            _ => [0.0; 3],
        }
    }
}

/// Render photograph `index` at `w × h`: area-averaged when shrinking, bilinear when enlarging, in
/// linear light, as linear Rec.2020. A damaged embedded file renders as flat mid-grey (a test
/// guards that it never happens).
pub(crate) fn render(index: u32, w: usize, h: usize, parallel: bool) -> Rgb32f {
    let mut img = Rgb32f::new(w, h);
    if w == 0 || h == 0 {
        return img;
    }
    let to2020 = SRGB.to_space(&REC2020).to_f32();
    let convert = |c: [f32; 3]| {
        [
            (to2020[0][0] * c[0] + to2020[0][1] * c[1] + to2020[0][2] * c[2]).max(0.0),
            (to2020[1][0] * c[0] + to2020[1][1] * c[1] + to2020[1][2] * c[2]).max(0.0),
            (to2020[2][0] * c[0] + to2020[2][1] * c[1] + to2020[2][2] * c[2]).max(0.0),
        ]
    };
    let Some(src) = get(index) else {
        img.data.iter_mut().for_each(|p| *p = convert([0.18; 3]));
        return img;
    };
    let t = lut();
    let (sx, sy) = (src.width as f32 / w as f32, src.height as f32 / h as f32);
    let enlarge = sx <= 1.0 && sy <= 1.0;
    let paint = |(y, row): (usize, &mut [[f32; 3]])| {
        for (x, px) in row.iter_mut().enumerate() {
            let c = if enlarge {
                // bilinear at the pixel centre
                let fx = ((x as f32 + 0.5) * sx - 0.5).max(0.0);
                let fy = ((y as f32 + 0.5) * sy - 0.5).max(0.0);
                let (x0, y0) = (fx as usize, fy as usize);
                let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
                let (a, b) = (src.linear(x0, y0, t), src.linear(x0 + 1, y0, t));
                let (c, d) = (src.linear(x0, y0 + 1, t), src.linear(x0 + 1, y0 + 1, t));
                let mut o = [0.0; 3];
                for k in 0..3 {
                    let top = a[k] + (b[k] - a[k]) * tx;
                    let bot = c[k] + (d[k] - c[k]) * tx;
                    o[k] = top + (bot - top) * ty;
                }
                o
            } else {
                // box average over the source pixels this one covers
                let (x0, y0) = ((x as f32 * sx) as usize, (y as f32 * sy) as usize);
                let x1 = (((x + 1) as f32 * sx).ceil() as usize).max(x0 + 1);
                let y1 = (((y + 1) as f32 * sy).ceil() as usize).max(y0 + 1);
                let mut acc = [0.0f32; 3];
                for yy in y0..y1 {
                    for xx in x0..x1 {
                        let p = src.linear(xx, yy, t);
                        acc = [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]];
                    }
                }
                let n = ((x1 - x0) * (y1 - y0)) as f32;
                [acc[0] / n, acc[1] / n, acc[2] / n]
            };
            *px = convert(c);
        }
    };
    if parallel {
        img.data.par_chunks_mut(w).enumerate().for_each(paint);
    } else {
        img.data.chunks_mut(w).enumerate().for_each(paint);
    }
    img
}
