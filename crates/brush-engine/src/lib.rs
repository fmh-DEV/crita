//! crita-brush-engine — núcleo do brush engine do Crita.
//!
//! Algoritmos portados do Krita (libs/brush + plugins/paintops), desacoplados
//! de Qt/KisPaintDevice. GPL-3.0-or-later, mesma licença do Krita.
//!
//! Organização: UMA engine por paint op do Krita (ver `paintops`), cada uma
//! com o `paintopid` equivalente — é o que preserva a compatibilidade com
//! presets .kpp. Não misturar comportamentos entre engines.
//!
//! Nada aqui conhece janela, arquivo ou GPU: tudo opera sobre o trait `Surface`.

pub mod blend;
pub mod mask;
pub mod paintops;
pub mod pen;
pub mod preset;
pub mod surface;

pub use blend::{mix, src_over};
pub use mask::AutoBrushMask;
pub use paintops::{
    factory_for, registry, ColorSmudgeBrush, ColorSmudgeParams, PaintOp, PaintOpFactory,
    PixelBrush, PixelBrushParams, StrokeState,
};
pub use pen::{PenSample, TipType};
pub use preset::BrushPreset;
pub use surface::{Image, Rgba, Surface};
