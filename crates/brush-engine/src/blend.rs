//! Compositing e mistura de cores (extraído de libs/pigment do Krita).

use crate::surface::Rgba;

/// Porter-Duff src-over com straight alpha.
pub fn src_over(dst: Rgba, src: Rgba) -> Rgba {
    let a = src.a + dst.a * (1.0 - src.a);
    if a <= f32::EPSILON {
        return Rgba::TRANSPARENT;
    }
    Rgba {
        r: (src.r * src.a + dst.r * dst.a * (1.0 - src.a)) / a,
        g: (src.g * src.a + dst.g * dst.a * (1.0 - src.a)) / a,
        b: (src.b * src.a + dst.b * dst.a * (1.0 - src.a)) / a,
        a,
    }
}

/// Mistura linear (usada pelo color smudge e pelo depósito por pixel).
pub fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}
