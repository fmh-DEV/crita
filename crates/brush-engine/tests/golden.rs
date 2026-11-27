//! Golden test: renderiza um traço determinístico e compara byte a byte
//! com tests/golden/stroke.ppm. Atualizar a referência (apenas de propósito!):
//!
//!     UPDATE_GOLDEN=1 cargo test -p crita-brush-engine --test golden
//!
//! Mudou a imagem sem querer? Bug no engine. Mudou por decisão? Regenere e
//! commite o novo ppm JUNTO com a explicação no commit.

use crita_brush_engine::{
    ColorSmudgeBrush, ColorSmudgeParams, Image, PaintOp, PenSample, PixelBrush, PixelBrushParams,
    Rgba,
};

fn to_ppm(img: &Image) -> Vec<u8> {
    let mut out = format!("P6\n{} {}\n255\n", img.width, img.height).into_bytes();
    for px in &img.data {
        out.push((px.r.clamp(0.0, 1.0) * 255.0) as u8);
        out.push((px.g.clamp(0.0, 1.0) * 255.0) as u8);
        out.push((px.b.clamp(0.0, 1.0) * 255.0) as u8);
    }
    out
}

// 6.2831 (não TAU) é de propósito: o golden foi gerado com esse valor e deve
// continuar byte-idêntico; mudar a constante exigiria regenerar a referência.
#[allow(clippy::approx_constant)]
#[test]
fn stroke_matches_golden() {
    let mut img = Image::new(256, 256, Rgba::WHITE);

    // traço 1: pixel brush vermelho com taper de pressão (paint op "defaultbrush")
    let mut brush = PixelBrush::new(PixelBrushParams {
        size: 40.0,
        hardness: 0.7,
        opacity: 0.9,
        spacing: 0.15,
        color: Rgba::rgb(0.85, 0.1, 0.1),
    });
    for i in 0..=100 {
        let t = i as f32 / 100.0;
        let x = 20.0 + t * 216.0;
        let y = 128.0 + (t * 6.2831).sin() * 60.0;
        let pressure = (1.0 - t * 0.8).clamp(0.2, 1.0);
        brush.line_to(&mut img, &PenSample { pressure, ..PenSample::mouse(x, y) });
    }

    // traço 2: color smudge azul sobre o vermelho (paint op "colorsmudge")
    let mut brush2 = ColorSmudgeBrush::new(ColorSmudgeParams {
        color: Rgba::rgb(0.1, 0.2, 0.9),
        smudge: 0.7,
        smudge_mix: 0.4,
        size: 30.0,
        ..ColorSmudgeParams::default()
    });
    for i in 0..=100 {
        let t = i as f32 / 100.0;
        brush2.line_to(&mut img, &PenSample::mouse(128.0 - t * 100.0, 40.0 + t * 176.0));
    }

    let ppm = to_ppm(&img);
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/stroke.ppm");

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(path, &ppm).expect("gravar golden");
        println!("golden atualizado em {path}");
        return;
    }

    let expected = std::fs::read(path).expect(
        "golden não existe; rode com UPDATE_GOLDEN=1 para gerar e commite o arquivo",
    );
    assert_eq!(ppm.len(), expected.len(), "tamanho do ppm mudou");
    assert!(
        ppm == expected,
        "imagem diferente do golden: o engine mudou de comportimento"
    );
}
