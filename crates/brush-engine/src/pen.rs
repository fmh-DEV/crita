//! Amostra de entrada da caneta, normalizada e independente de plataforma
//! (ADR-0003, item 4). A posição vem da camada de janelas; pressure/tilt/
//! rotation/tip_type vêm do feed nativo (crates/input, futuro). Eixos
//! sintéticos (velocity, direction) são computados na camada de normalização
//! de input, NÃO nas engines (ADR-0002).

/// Ponta em contato com a superfície (canetas reportam a borracha como
/// ponta própria — o Krita trata como outro device de pintura).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipType {
    Pen,
    Eraser,
}

/// Um ponto de entrada já normalizado. Paint ops consomem isto — nunca
/// eventos de toolkit ou de plataforma.
#[derive(Clone, Copy, Debug)]
pub struct PenSample {
    /// Posição no canvas, em px (vem da camada de janelas).
    pub pos: (f32, f32),
    /// Pressão normalizada 0..1 (mouse = 1.0, ver ADR-0003).
    pub pressure: f32,
    /// Tilt normalizado -1..1 por eixo (convenção GIMP; conversões nas
    /// fronteiras: .kpp/Krita -60..60°, W3C -90..90°).
    pub tilt: (f32, f32),
    /// Rotação da caneta em torno do próprio eixo, em radianos.
    pub rotation: f32,
    pub tip_type: TipType,
}

impl PenSample {
    /// Amostra de mouse/trackpad: pressão total, sem tilt nem rotação.
    pub fn mouse(x: f32, y: f32) -> PenSample {
        PenSample {
            pos: (x, y),
            pressure: 1.0,
            tilt: (0.0, 0.0),
            rotation: 0.0,
            tip_type: TipType::Pen,
        }
    }
}
