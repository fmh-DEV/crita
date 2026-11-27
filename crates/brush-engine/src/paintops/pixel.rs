//! Pixel Brush — port de plugins/paintops/defaultpaintops (KisBrushOp, Krita).
//!
//! Comportamento do Krita, sem extensões: carimba dabs da máscara ao longo do
//! traço com spacing × diâmetro e compõe cada dab em src-over. A opacidade
//! escala o alfa da máscara (Krita: KisFixedPaintDevice da dab multiplicado
//! pela opacidade do paint op). SEM smudge — isso é outro paint op
//! (`paintops::colorsmudge`).

use crate::mask::AutoBrushMask;
use crate::paintops::{PaintOp, PaintOpFactory, StrokeState};
use crate::pen::PenSample;
use crate::surface::{Rgba, Surface};

/// Entrada desta engine no registry (ADR-0006).
pub const FACTORY: PaintOpFactory = PaintOpFactory {
    id: "defaultbrush",
    name: "Pixel Brush",
    create: || Box::new(PixelBrush::new(PixelBrushParams::default())),
};

/// Parâmetros do Pixel Brush (subset dos que o preset .kpp expõe para
/// `paintopid="defaultbrush"`). Crescer aqui = seguir o .kpp, não inventar.
#[derive(Clone, Copy, Debug)]
pub struct PixelBrushParams {
    pub size: f32,     // diâmetro em px (Krita: "Size")
    pub hardness: f32, // 0..1 (Krita: "Hardness", da máscara auto brush)
    pub opacity: f32,  // 0..1 (Krita: "Opacity")
    pub spacing: f32,  // fração do diâmetro (Krita: "Spacing"; 0.25 = dab a cada ¼)
    pub color: Rgba,   // cor de foreground (vem do seletor, não do preset)
}

impl Default for PixelBrushParams {
    fn default() -> Self {
        PixelBrushParams {
            size: 24.0,
            hardness: 0.8,
            opacity: 1.0,
            spacing: 0.15,
            color: Rgba::BLACK,
        }
    }
}

pub struct PixelBrush {
    pub params: PixelBrushParams,
    state: StrokeState,
}

impl PixelBrush {
    pub fn new(params: PixelBrushParams) -> Self {
        PixelBrush { params, state: StrokeState::default() }
    }

    fn step(params: &PixelBrushParams, pressure: f32) -> f32 {
        (params.size * pressure * params.spacing).max(1.0)
    }

    /// Carimba um dab em (x, y): src-over da cor do pincel com alfa
    /// máscara × opacidade, pixel a pixel.
    pub fn dab<S: ?Sized + Surface>(&self, surface: &mut S, x: f32, y: f32, pressure: f32) {
        Self::dab_with(&self.params, surface, x, y, pressure);
    }

    fn dab_with<S: ?Sized + Surface>(params: &PixelBrushParams, surface: &mut S, x: f32, y: f32, pressure: f32) {
        let radius = (params.size * pressure * 0.5).max(0.5);
        let mask = AutoBrushMask::new(radius, params.hardness);
        let src = params.color;

        let left = (x - mask.radius) as i32;
        let top = (y - mask.radius) as i32;
        for my in 0..mask.size {
            for mx in 0..mask.size {
                let px = left + mx as i32;
                let py = top + my as i32;
                if px < 0 || py < 0 || px >= surface.width() as i32 || py >= surface.height() as i32 {
                    continue;
                }
                let a = mask.at(mx, my) * params.opacity * src.a;
                if a <= f32::EPSILON {
                    continue;
                }
                surface.blend_pixel(px as usize, py as usize, Rgba { a, ..src });
            }
        }
    }
}

impl PaintOp for PixelBrush {
    fn id(&self) -> &'static str {
        FACTORY.id
    }

    fn reset_stroke(&mut self) {
        self.state.reset();
    }

    /// Continua o traço até `sample.pos`, posicionando dabs com o spacing
    /// configurado.
    fn line_to(&mut self, surface: &mut dyn Surface, sample: &PenSample) {
        let params = self.params;
        let pressure = sample.pressure;
        let step = Self::step(&params, pressure);
        let (x, y) = sample.pos;
        self.state.line_to(step, x, y, |dx, dy| Self::dab_with(&params, surface, dx, dy, pressure));
    }
}
