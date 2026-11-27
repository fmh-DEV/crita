//! Color Smudge — SERÁ o port de plugins/paintops/colorsmudge
//! (KisColorSmudgeOp, Krita).
//!
//! ⚠ WIP — NÃO é o port fiel ainda. O código abaixo é a semente que existia
//! embutida na antiga PixelBrush híbrida (modelo "coloring" simplificado com
//! depósito por pixel). O port fiel precisa trazer do Krita:
//!   - os modos Dulling e Smearing (KisColorSmudgeOp::paint Op trata os dois);
//!   - "SmudgeLength" (comprimento do arraste) e "ColorRate" (taxa da cor do
//!     pincel) com as curvas de sensor;
//!   - o "smudge radius" separado do raio do dab;
//!   - a amostragem da tinta via KisPaintDevice (hoje: average_color simples).
//!
//! Até lá, NÃO usar como referência de compatibilidade com presets .kpp.

use crate::blend::mix;
use crate::mask::AutoBrushMask;
use crate::paintops::{PaintOp, PaintOpFactory, StrokeState};
use crate::pen::PenSample;
use crate::surface::{Rgba, Surface};

/// Entrada desta engine no registry (ADR-0006).
pub const FACTORY: PaintOpFactory = PaintOpFactory {
    id: "colorsmudge",
    name: "Color Smudge",
    create: || Box::new(ColorSmudgeBrush::new(ColorSmudgeParams::default())),
};

/// Parâmetros do Color Smudge (nomes provisórios; o port fiel adota os nomes
/// dos campos do .kpp: SmudgeLength, ColorRate, SmudgeRadius...).
#[derive(Clone, Copy, Debug)]
pub struct ColorSmudgeParams {
    pub size: f32,
    pub hardness: f32,
    pub opacity: f32,
    pub spacing: f32,
    pub color: Rgba,
    pub smudge: f32,      // 0 = cor pura do pincel; >0 = arraste (Krita: "SmudgeLength")
    pub smudge_mix: f32,  // fração da cor do pincel no depósito (Krita: "ColorRate")
    pub deposit_rate: f32, // taxa de pull por dab (1.0 substituiria; 0.35 mistura)
}

impl Default for ColorSmudgeParams {
    fn default() -> Self {
        ColorSmudgeParams {
            size: 40.0,
            hardness: 0.6,
            opacity: 1.0,
            spacing: 0.15,
            color: Rgba::BLACK,
            smudge: 0.7,
            smudge_mix: 0.4,
            deposit_rate: 0.35,
        }
    }
}

/// Estado por traço: espaçamento de dabs + tinta atualmente carregada
/// (o "pincel sujo" do smudge). Interno à engine — chamadores veem só o
/// trait `PaintOp`.
#[derive(Default)]
struct ColorSmudgeState {
    stroke: StrokeState,
    smudge_color: Option<Rgba>,
}

impl ColorSmudgeState {
    fn reset(&mut self) {
        self.stroke.reset();
        self.smudge_color = None;
    }
}

pub struct ColorSmudgeBrush {
    pub params: ColorSmudgeParams,
    state: ColorSmudgeState,
}

impl ColorSmudgeBrush {
    pub fn new(params: ColorSmudgeParams) -> Self {
        ColorSmudgeBrush { params, state: ColorSmudgeState::default() }
    }

    fn step(params: &ColorSmudgeParams, pressure: f32) -> f32 {
        (params.size * pressure * params.spacing).max(1.0)
    }

    /// Cor depositada neste dab (tinta arrastada + cor do pincel, se smudge).
    fn deposit_color<S: ?Sized + Surface>(params: &ColorSmudgeParams, surface: &S, smudge_color: &mut Option<Rgba>, x: f32, y: f32, radius: f32) -> Rgba {
        if params.smudge <= 0.0 {
            return params.color;
        }
        let picked = surface.average_color(x, y, radius);
        let base = match *smudge_color {
            Some(c) => c,
            None => picked, // estado nasce da tinta capturada: sem "lama" da cor do pincel
        };
        // estado segue o canvas com inércia (smudge length)
        let dragged = mix(base, picked, params.smudge);
        *smudge_color = Some(dragged);
        // cor depositada = tinta arrastada + cor do pincel (modo coloring)
        mix(dragged, params.color, params.smudge_mix)
    }

    fn dab_inner<S: ?Sized + Surface>(params: &ColorSmudgeParams, surface: &mut S, smudge_color: &mut Option<Rgba>, x: f32, y: f32, pressure: f32) {
        let radius = (params.size * pressure * 0.5).max(0.5);
        let mask = AutoBrushMask::new(radius, params.hardness);
        let deposit = Self::deposit_color(params, surface, smudge_color, x, y, radius);

        let left = (x - mask.radius) as i32;
        let top = (y - mask.radius) as i32;
        for my in 0..mask.size {
            for mx in 0..mask.size {
                let px = left + mx as i32;
                let py = top + my as i32;
                if px < 0 || py < 0 || px >= surface.width() as i32 || py >= surface.height() as i32 {
                    continue;
                }
                let w = mask.at(mx, my);
                if w < 0.02 {
                    continue;
                }
                let k = (w * params.opacity * params.deposit_rate).min(1.0);
                if k <= f32::EPSILON {
                    continue;
                }
                let dst = surface.get(px as usize, py as usize);
                surface.set_pixel(px as usize, py as usize, mix(dst, deposit, k));
            }
        }
    }

    /// Carimba um dab em (x, y): cada pixel sob a máscara é puxado
    /// (lerp) em direção à cor depositada — mistura de dois lados.
    pub fn dab<S: ?Sized + Surface>(&mut self, surface: &mut S, x: f32, y: f32, pressure: f32) {
        Self::dab_inner(&self.params, surface, &mut self.state.smudge_color, x, y, pressure);
    }
}

impl PaintOp for ColorSmudgeBrush {
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
        let ColorSmudgeState { stroke, smudge_color } = &mut self.state;
        stroke.line_to(step, x, y, |dx, dy| Self::dab_inner(&params, surface, smudge_color, dx, dy, pressure));
    }
}
