//! Paint ops — uma engine por paint op do Krita, nunca misturadas.
//!
//! Cada módulo é o port fiel de um paint op do Krita
//! (plugins/paintops/<nome>), identificado pelo MESMO `paintopid` que os
//! presets .kpp usam. É isso que garante compatibilidade: um preset do Krita
//! carrega a engine equivalente e produz o mesmo traço.
//!
//! O registry (`registry()`/`factory_for()`) é o único ponto de registro:
//! adicionar engine = 1 módulo novo + 1 entrada no slice (ADR-0006). Engines
//! de outros ecossistemas (Photoshop via adaptador PhotoCraft, eventual CSP)
//! entram pelo mesmo registry, cada uma lendo seu formato de preset nativo.
//!
//! Roadmap de ports (um por vez, cada um com golden test próprio):
//!   - `pixel`       — defaultpaintops (KisBrushOp)          [portado]
//!   - `colorsmudge` — colorsmudge (KisColorSmudgeOp)        [stub WIP]
//!   - mypaint, sketch, spray...                              [a portar]

pub mod colorsmudge;
pub mod pixel;

use crate::pen::PenSample;
use crate::surface::Surface;

pub use colorsmudge::{ColorSmudgeBrush, ColorSmudgeParams};
pub use pixel::{PixelBrush, PixelBrushParams};

/// Interface de uma paint op (Krita: `KisPaintOp`). Object-safe: chamadores
/// conhecem só `Box<dyn PaintOp>` — a engine concreta fica atrás do
/// `paintopid`. Cada instância carrega seu estado de traço interno, como o
/// paint op do Krita, criado por stroke. A factory é o registry
/// (`factory_for`); plugins dinâmicos não existem (ADR-0002, ADR-0006).
pub trait PaintOp {
    /// Identidade da engine: o `paintopid` nativo ("defaultbrush"...).
    /// Deve bater com o `id` da `FACTORY` que a criou.
    fn id(&self) -> &'static str;
    /// Reinicia o estado por traço (novo stroke).
    fn reset_stroke(&mut self);
    /// Continua o traço até `sample.pos` (Krita: paintLine com
    /// KisPaintInformation). Cada engine usa pressure/tilt/rotation conforme
    /// seus parâmetros.
    fn line_to(&mut self, surface: &mut dyn Surface, sample: &PenSample);
}

/// Factory de uma paint op no registry (Krita: plugin + factory do
/// KisPaintOpRegistry). Cada módulo de engine exporta sua `FACTORY` como
/// const; `registry()` é a lista completa e o único ponto de registro.
pub struct PaintOpFactory {
    /// Identidade da engine: o `paintopid` dos presets ("defaultbrush"...).
    pub id: &'static str,
    /// Nome de exibição na UI ("Pixel Brush").
    pub name: &'static str,
    /// Cria a engine com parâmetros default. Passa a receber as settings do
    /// preset quando a leitura de .kpp/.cbrush chegar (ADR-0006, item 4).
    pub create: fn() -> Box<dyn PaintOp>,
}

/// Todas as engines registradas, na ordem de exibição da UI.
pub fn registry() -> &'static [PaintOpFactory] {
    &[pixel::FACTORY, colorsmudge::FACTORY]
}

/// Resolve o `paintopid` de um preset. `None` = engine ainda não portada —
/// o preset não deve carregar como outra engine (ADR-0002).
pub fn factory_for(id: &str) -> Option<&'static PaintOpFactory> {
    registry().iter().find(|f| f.id == id)
}

/// Estado do espaçamento de dabs ao longo de um traço — o equivalente ao
/// controle de spacing do KisPaintOp (Krita). Compartilhado por todas as
/// engines; estados específicos (ex: tinta carregada do smudge) ficam nas
/// structs de estado de cada paint op.
pub struct StrokeState {
    last: Option<(f32, f32)>,
    acc: f32,
}

impl Default for StrokeState {
    fn default() -> Self {
        StrokeState { last: None, acc: 0.0 }
    }
}

impl StrokeState {
    pub fn reset(&mut self) {
        self.last = None;
        self.acc = 0.0;
    }

    /// Avança o traço até (x, y), disparando `dab` a cada `step` px
    /// (Krita: spacing é fração do diâmetro; o step já vem calculado).
    pub fn line_to(&mut self, step: f32, x: f32, y: f32, mut dab: impl FnMut(f32, f32)) {
        match self.last {
            None => dab(x, y),
            Some((lx, ly)) => {
                let dx = x - lx;
                let dy = y - ly;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 1e-6 {
                    return;
                }
                let (ux, uy) = (dx / dist, dy / dist);
                let first = step - self.acc;
                if dist >= first {
                    let mut travelled = first;
                    let (mut px, mut py) = (lx + ux * first, ly + uy * first);
                    dab(px, py);
                    while travelled + step <= dist {
                        px += ux * step;
                        py += uy * step;
                        travelled += step;
                        dab(px, py);
                    }
                    self.acc = dist - travelled;
                } else {
                    self.acc += dist;
                }
            }
        }
        self.last = Some((x, y));
    }
}
