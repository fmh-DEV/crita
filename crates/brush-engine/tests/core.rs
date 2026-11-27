use crita_brush_engine::{
    factory_for, mix, registry, src_over, AutoBrushMask, BrushPreset, ColorSmudgeBrush,
    ColorSmudgeParams, Image, PixelBrush, PixelBrushParams, Rgba, Surface,
};

const EPS: f32 = 1e-5;

#[test]
fn mask_center_opaque_edge_transparent() {
    let m = AutoBrushMask::new(10.0, 0.8);
    assert!((m.at(10, 10) - 1.0).abs() < EPS);
    assert_eq!(m.at(0, 0), 0.0);
    let inner = 8.0f32;
    let d = (inner + 10.0) / 2.0;
    let v = m.at((10.0 + d).round() as usize, 10);
    assert!(v > 0.1 && v < 0.9, "falloff parcial esperado, obteve {v}");
}

#[test]
fn blend_semantics() {
    let r = src_over(Rgba::rgb(0.0, 0.0, 1.0), Rgba::rgb(1.0, 0.0, 0.0));
    assert!((r.r - 1.0).abs() < EPS && (r.a - 1.0).abs() < EPS);
    let m = mix(Rgba::rgb(1.0, 0.0, 0.0), Rgba::rgb(0.0, 0.0, 1.0), 0.5);
    assert!((m.r - 0.5).abs() < EPS && (m.b - 0.5).abs() < EPS);
}

#[test]
fn dab_pinta_centro() {
    let mut img = Image::new(64, 64, Rgba::WHITE);
    let brush = PixelBrush::new(PixelBrushParams {
        color: Rgba::rgb(1.0, 0.0, 0.0),
        ..Default::default()
    });
    brush.dab(&mut img, 32.0, 32.0, 1.0);
    let c = img.get(32, 32);
    assert!(c.r > 0.9 && c.g < 0.2, "centro do dab deveria ser vermelho: {c:?}");
}

#[test]
fn pixel_brush_nao_arrasta_tinta() {
    // pixel brush puro: pintar azul sobre vermelho cobre com azul (src-over),
    // nunca mistura com o que está no canvas
    let mut img = Image::new(64, 64, Rgba::rgb(0.9, 0.1, 0.1));
    let brush = PixelBrush::new(PixelBrushParams {
        color: Rgba::rgb(0.1, 0.2, 0.9),
        hardness: 1.0,
        ..Default::default()
    });
    brush.dab(&mut img, 32.0, 32.0, 1.0);
    let c = img.get(32, 32);
    assert!(c.b > 0.8 && c.r < 0.2, "pixel brush deve cobrir, não misturar: {c:?}");
}

#[test]
fn smudge_mistura_cor_do_canvas_e_do_pincel() {
    // canvas vermelho, pincel azul com smudge: primeiro dab sobre vermelho
    // deve depositar algo entre vermelho e azul (nunca cobrir de uma vez)
    let mut img = Image::new(64, 64, Rgba::rgb(0.9, 0.1, 0.1));
    let mut brush = ColorSmudgeBrush::new(ColorSmudgeParams {
        color: Rgba::rgb(0.1, 0.2, 0.9),
        smudge: 0.7,
        smudge_mix: 0.4,
        hardness: 1.0,
        ..Default::default()
    });
    brush.dab(&mut img, 32.0, 32.0, 1.0);
    let c = img.get(32, 32);
    assert!(
        c.r > 0.4 && c.b > 0.2,
        "esperava mistura roxa (r e b altos), obteve {c:?}"
    );
    assert!(
        c.r < 0.9,
        "esperava que o vermelho original participasse (não cobre 100%), obteve {c:?}"
    );
}

#[test]
fn paintop_ids_batem_com_krita() {
    // os ids do registry são os paintopids dos presets .kpp (ADR-0002)
    assert!(factory_for("defaultbrush").is_some());
    assert!(factory_for("colorsmudge").is_some());
    assert!(factory_for("mypaintbrush").is_none(), "engine ainda não portada");
}

#[test]
fn registry_factories_produzem_engine_com_mesmo_id() {
    // toda factory registrada produz uma engine cuja identidade bate com a
    // entrada — é o que garante que preset carrega a engine certa (ADR-0006)
    for f in registry() {
        let op = (f.create)();
        assert_eq!(op.id(), f.id, "factory {} produziu engine com id divergente", f.id);
    }
}

#[test]
fn presets_exist() {
    let p = BrushPreset::pixel_default();
    assert_eq!(p.name, "Pixel");
    assert_eq!(p.paintop_id, "defaultbrush");
    let s = BrushPreset::smudge_default();
    assert_eq!(s.paintop_id, "colorsmudge");
}

#[test]
fn factory_cria_engine_pelo_registry() {
    // Krita: KisPaintOpRegistry::factory(paintopid) — o chamador recebe só
    // Box<dyn PaintOp> e a identidade vem do paintopid do preset
    let op = BrushPreset::pixel_default().create_paint_op().expect("pixel registrado");
    assert_eq!(op.id(), "defaultbrush");
    let op = BrushPreset::smudge_default().create_paint_op().expect("smudge registrado");
    assert_eq!(op.id(), "colorsmudge");
}
