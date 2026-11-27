//! Preset de brush. Modelo mínimo da fase 1 — formatos de serialização
//! (.cbrush nativo, leitura direta de .kpp do Krita) entram na fase seguinte:
//! o preset passa a carregar as settings lidas do arquivo, e cada família de
//! engine lê seu formato nativo (.kpp, .abr... — ver ADR-0006, item 4).
//!
//! Um preset aponta para UMA paint op pelo `paintop_id` (o paintopid do
//! .kpp). Presets de paint ops não registradas falham explicitamente
//! (`create_paint_op` → None), nunca carregam como outra engine (ADR-0002).
//! Parâmetros tipados por engine ficam no módulo da engine — a edição
//! tipada (brush editor) é da camada que conhece a engine, não do preset.

use crate::paintops::{self, PaintOp};

#[derive(Clone, Debug)]
pub struct BrushPreset {
    pub name: String,
    /// paintopid da engine ("defaultbrush", "colorsmudge"...).
    pub paintop_id: String,
}

impl BrushPreset {
    /// Pincel de tinta padrão (o "Pixel Brush" do Krita, `defaultbrush`).
    pub fn pixel_default() -> BrushPreset {
        BrushPreset { name: "Pixel".into(), paintop_id: "defaultbrush".into() }
    }

    /// Color smudge padrão (`colorsmudge`). Parâmetros provisórios até o
    /// port fiel do KisColorSmudgeOp (ver paintops::colorsmudge).
    pub fn smudge_default() -> BrushPreset {
        BrushPreset { name: "Smudge".into(), paintop_id: "colorsmudge".into() }
    }

    /// Cria a engine do preset via registry (Krita:
    /// KisPaintOpRegistry::factory(paintopid)). Chamadores conhecem só o
    /// trait `PaintOp`. None = engine não registrada.
    pub fn create_paint_op(&self) -> Option<Box<dyn PaintOp>> {
        paintops::factory_for(&self.paintop_id).map(|f| (f.create)())
    }
}
