//! crita-playground — canvas mínimo (eframe/egui + wgpu) pra sentir o brush engine.
//! Mede o tempo do line_to: é o começo do nosso contrato de latência
//! (pen-to-pixel < ~30 ms em telas 120-160 Hz — ver ADR-0001 e golden rule 3).

use crita_brush_engine::{Image, PaintOp, PenSample, PixelBrush, PixelBrushParams, Rgba};
use eframe::egui;
use std::time::{Duration, Instant};

const W: usize = 1024;
const H: usize = 640;

struct State {
    canvas: Image,
    brush: Option<Box<dyn PaintOp>>,
    undo_stack: Vec<Vec<Rgba>>,
    line_ms_sum: f32,
    line_ms_worst: f32,
    line_n: u32,
}

impl State {
    fn new() -> State {
        State {
            canvas: Image::new(W, H, Rgba::WHITE),
            brush: None,
            undo_stack: Vec::new(),
            line_ms_sum: 0.0,
            line_ms_worst: 0.0,
            line_n: 0,
        }
    }

    fn snapshot(&mut self) {
        self.undo_stack.push(self.canvas.data.clone());
        if self.undo_stack.len() > 8 {
            self.undo_stack.remove(0);
        }
    }
}

struct PlaygroundApp {
    state: State,
    texture: egui::TextureHandle,
    dirty: bool,
    brush_size: f32,
    brush_hardness: f32,
    brush_opacity: f32,
    brush_spacing: f32,
    brush_color: egui::Color32,
    stats: String,
    stats_since: Instant,
}

impl PlaygroundApp {
    fn new(cc: &eframe::CreationContext) -> PlaygroundApp {
        let state = State::new();
        let texture = cc.egui_ctx.load_texture(
            "canvas",
            color_image(&state.canvas),
            egui::TextureOptions::LINEAR,
        );
        PlaygroundApp {
            state,
            texture,
            dirty: false,
            brush_size: 24.0,
            brush_hardness: 80.0,
            brush_opacity: 100.0,
            brush_spacing: 15.0,
            brush_color: egui::Color32::from_rgb(0x14, 0x14, 0x1a),
            stats: "pinte com o mouse".to_string(),
            stats_since: Instant::now(),
        }
    }

    // configuração tipada por engine (o brush editor conhece a engine que
    // edita, como os option widgets do Krita — ver ADR-0006, item 2)
    fn current_brush(&self) -> Box<dyn PaintOp> {
        Box::new(PixelBrush::new(PixelBrushParams {
            size: self.brush_size,
            hardness: self.brush_hardness / 100.0,
            opacity: self.brush_opacity / 100.0,
            spacing: self.brush_spacing / 100.0,
            color: Rgba::rgb(
                self.brush_color.r() as f32 / 255.0,
                self.brush_color.g() as f32 / 255.0,
                self.brush_color.b() as f32 / 255.0,
            ),
        }))
    }

    fn panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("panel").exact_size(230.0).show(ui, |ui| {
            ui.add_space(10.0);
            ui.heading("crita playground");
            ui.add_space(6.0);
            ui.add(egui::Slider::new(&mut self.brush_size, 2.0..=120.0).text("tamanho"));
            ui.add(egui::Slider::new(&mut self.brush_hardness, 0.0..=100.0).text("dureza %"));
            ui.add(egui::Slider::new(&mut self.brush_opacity, 1.0..=100.0).text("opacidade %"));
            ui.add(egui::Slider::new(&mut self.brush_spacing, 2.0..=100.0).text("espaçamento %"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                for c in [
                    egui::Color32::from_rgb(0x14, 0x14, 0x1a),
                    egui::Color32::from_rgb(0xc0, 0x39, 0x2b),
                    egui::Color32::from_rgb(0x2b, 0x50, 0xc0),
                    egui::Color32::from_rgb(0x1e, 0x8a, 0x4c),
                    egui::Color32::from_rgb(0xe6, 0x7e, 0x22),
                ] {
                    let b = egui::Button::new("").fill(c).min_size(egui::vec2(26.0, 26.0));
                    if ui.add(b).clicked() {
                        self.brush_color = c;
                    }
                }
            });
            ui.color_edit_button_srgba(&mut self.brush_color);
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button("desfazer").clicked() {
                    if let Some(snap) = self.state.undo_stack.pop() {
                        self.state.canvas.data = snap;
                        self.dirty = true;
                    }
                }
                if ui.button("limpar").clicked() {
                    self.state.snapshot();
                    self.state.canvas = Image::new(W, H, Rgba::WHITE);
                    self.state.brush = None;
                    self.dirty = true;
                }
            });
            ui.add_space(6.0);
            ui.label(&self.stats);
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            let avail = ui.available_size();
            let scale = (avail.x / W as f32).min(avail.y / H as f32).max(0.01);
            let size = egui::vec2(W as f32 * scale, H as f32 * scale);
            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());
            ui.painter().image(
                self.texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            let to_canvas = |pos: egui::Pos2| {
                (
                    (pos.x - rect.min.x) * W as f32 / rect.width().max(1.0),
                    (pos.y - rect.min.y) * H as f32 / rect.height().max(1.0),
                )
            };

            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    self.state.snapshot();
                    let mut brush = self.current_brush();
                    let (cx, cy) = to_canvas(pos);
                    brush.line_to(&mut self.state.canvas, &PenSample::mouse(cx, cy));
                    self.state.brush = Some(brush);
                    self.dirty = true;
                }
            }
            if response.dragged() {
                if let (Some(brush), Some(pos)) =
                    (self.state.brush.as_mut(), response.interact_pointer_pos())
                {
                    let (cx, cy) = to_canvas(pos);
                    let t0 = Instant::now();
                    brush.line_to(&mut self.state.canvas, &PenSample::mouse(cx, cy));
                    let ms = t0.elapsed().as_secs_f32() * 1000.0;
                    self.state.line_ms_sum += ms;
                    self.state.line_n += 1;
                    if ms > self.state.line_ms_worst {
                        self.state.line_ms_worst = ms;
                    }
                    self.dirty = true;
                }
            }
            if response.drag_stopped() {
                self.state.brush = None;
            }
        });
    }
}

impl eframe::App for PlaygroundApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.panel(ui);
        self.canvas(ui);

        // upload só quando o canvas mudou (o compositor real fará dirty-rect;
        // aqui a textura inteira só sobe nos frames em que houve pintura)
        if self.dirty {
            self.texture
                .set(color_image(&self.state.canvas), egui::TextureOptions::LINEAR);
            self.dirty = false;
        }

        if self.stats_since.elapsed() >= Duration::from_millis(500) {
            let st = &mut self.state;
            let avg = if st.line_n > 0 { st.line_ms_sum / st.line_n as f32 } else { 0.0 };
            self.stats = format!(
                "engine: media {:.2} ms/linha | pior {:.2} ms | n={}\n(meta < 8 ms em 120 Hz; undo: {} niveis)",
                avg,
                st.line_ms_worst,
                st.line_n,
                st.undo_stack.len()
            );
            st.line_ms_sum = 0.0;
            st.line_ms_worst = 0.0;
            st.line_n = 0;
            self.stats_since = Instant::now();
        }
        ui.ctx().request_repaint_after(Duration::from_millis(500));
    }
}

fn color_image(canvas: &Image) -> egui::ColorImage {
    let pixels = canvas
        .data
        .iter()
        .map(|px| {
            let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
            egui::Color32::from_rgb(c(px.r), c(px.g), c(px.b))
        })
        .collect();
    egui::ColorImage::new([W, H], pixels)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "crita playground",
        options,
        Box::new(|cc| Ok(Box::new(PlaygroundApp::new(cc)))),
    )
}
