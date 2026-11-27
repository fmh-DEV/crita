//! crita-render — compositor CPU de referência (oráculo).
//!
//! Fase 1: uma pilha de camadas opacas em src-over. O compositor wgpu virá
//! depois implementando o MESMO contrato, e os dois serão testados um contra
//! o outro (imagem igual => o wgpu é o culpado quando divergir).

use crita_brush_engine::{Image, Rgba, Surface};

pub struct Layer {
    pub image: Image,
    pub opacity: f32, // 0..1
    pub visible: bool,
}

impl Layer {
    pub fn new(image: Image) -> Layer {
        Layer { image, opacity: 1.0, visible: true }
    }
}

/// Compõe a pilha de camadas numa única imagem (fundo branco opaco).
pub fn composite(layers: &[Layer]) -> Image {
    let (w, h) = match layers.iter().find(|l| l.visible) {
        Some(l) => (l.image.width(), l.image.height()),
        None => return Image::new(1, 1, Rgba::WHITE),
    };
    let mut out = Image::new(w, h, Rgba::WHITE);
    for layer in layers.iter().filter(|l| l.visible) {
        for y in 0..h {
            for x in 0..w {
                let src = layer.image.get(x, y);
                let src = Rgba { a: src.a * layer.opacity, ..src };
                out.blend_pixel(x, y, src);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_applies_opacity() {
        let red = Image::new(4, 4, Rgba::rgb(1.0, 0.0, 0.0));
        let layer = Layer { image: red, opacity: 0.5, visible: true };
        let out = composite(&[layer]);
        let c = out.get(0, 0);
        // vermelho opaco a 50% sobre branco: r fica em 1.0 (ambos são 1.0), g/b vão a 0.5
        assert!((c.r - 1.0).abs() < 1e-6, "canal r permanece 1.0: {c:?}");
        assert!((c.g - 0.5).abs() < 1e-6, "canal g mistura com branco: {c:?}");
        assert!((c.b - 0.5).abs() < 1e-6, "canal b mistura com branco: {c:?}");
        assert!((c.a - 1.0).abs() < 1e-6, "alfa fica opaco sobre fundo branco: {c:?}");
    }

    #[test]
    fn invisible_layers_are_skipped() {
        let red = Image::new(2, 2, Rgba::rgb(1.0, 0.0, 0.0));
        let layer = Layer { image: red, opacity: 1.0, visible: false };
        let out = composite(&[layer]);
        let c = out.get(0, 0);
        assert_eq!(c, Rgba::WHITE);
    }
}
