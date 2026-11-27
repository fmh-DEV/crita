//! crita-playground — canvas mínimo (Slint desktop) pra sentir o brush engine.
//! Mede o tempo do line_to: é o começo do nosso contrato de latência
//! (pen-to-pixel < ~30 ms em telas 120-160 Hz — ver ADR-0001).
#![allow(unused_crate_dependencies)]

slint::include_modules!();

use crita_brush_engine::{Image, PaintOp, PenSample, PixelBrush, PixelBrushParams, Rgba};
use slint::{Image as SImage, SharedPixelBuffer};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

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

fn current_brush(ui: &AppWindow) -> Box<dyn PaintOp> {
    // configuração tipada por engine (o brush editor conhece a engine que
    // edita, como os option widgets do Krita — ver ADR-0006, item 2)
    Box::new(PixelBrush::new(PixelBrushParams {
        size: ui.get_brush_size(),
        hardness: ui.get_brush_hardness() / 100.0,
        opacity: ui.get_brush_opacity() / 100.0,
        spacing: ui.get_brush_spacing() / 100.0,
        color: Rgba::from_hex(&ui.get_brush_color()),
    }))
}

fn copy_to_buffer(canvas: &Image, buf: &mut SharedPixelBuffer<slint::Rgb8Pixel>) {
    let pixels = buf.make_mut_slice();
    for (i, px) in canvas.data.iter().enumerate() {
        pixels[i] = slint::Rgb8Pixel {
            r: (px.r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (px.g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (px.b.clamp(0.0, 1.0) * 255.0) as u8,
        };
    }
}

fn map_coords(x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    (x * W as f32 / w.max(1.0), y * H as f32 / h.max(1.0))
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = std::rc::Rc::new(AppWindow::new()?);
    let state = Rc::new(RefCell::new(State::new()));
    let buffer = Rc::new(RefCell::new(SharedPixelBuffer::<slint::Rgb8Pixel>::new(W as u32, H as u32)));

    // imagem inicial
    {
        let st = state.borrow();
        let mut b = buffer.borrow_mut();
        copy_to_buffer(&st.canvas, &mut b);
        ui.set_canvas_image(SImage::from_rgb8(b.clone()));
    }

    // pointer down: snapshot pra undo + primeiro dab
    {
        let state = state.clone();
        let buffer = buffer.clone();
        let ui_weak = ui.as_weak();
        ui.on_pointer_down(move |x, y, w, h| {
            let ui = ui_weak.upgrade().unwrap();
            let mut st = state.borrow_mut();
            st.snapshot();
            let mut brush = current_brush(&ui);
            let (cx, cy) = map_coords(x, y, w, h);
            brush.line_to(&mut st.canvas, &PenSample::mouse(cx, cy));
            st.brush = Some(brush);
            let mut b = buffer.borrow_mut();
            copy_to_buffer(&st.canvas, &mut b);
            ui.set_canvas_image(SImage::from_rgb8(b.clone()));
        });
    }

    // pointer move: pinta + mede tempo do line_to (parte do engine na latência)
    {
        let state = state.clone();
        let buffer = buffer.clone();
        let ui_weak = ui.as_weak();
        ui.on_pointer_move(move |x, y, w, h| {
            let ui = ui_weak.upgrade().unwrap();
            let t0 = Instant::now();
            {
                let mut st = state.borrow_mut();
                let st = &mut *st;
                if let Some(brush) = st.brush.as_mut() {
                    let (cx, cy) = map_coords(x, y, w, h);
                    brush.line_to(&mut st.canvas, &PenSample::mouse(cx, cy));
                    let ms = t0.elapsed().as_secs_f32() * 1000.0;
                    st.line_ms_sum += ms;
                    st.line_n += 1;
                    if ms > st.line_ms_worst {
                        st.line_ms_worst = ms;
                    }
                }
            }
            let st = state.borrow();
            let mut b = buffer.borrow_mut();
            copy_to_buffer(&st.canvas, &mut b);
            ui.set_canvas_image(SImage::from_rgb8(b.clone()));
        });
    }

    // pointer up: fecha stroke
    {
        let state = state.clone();
        ui.on_pointer_up(move || {
            state.borrow_mut().brush = None;
        });
    }

    // desfazer / limpar
    {
        let state = state.clone();
        let buffer = buffer.clone();
        let ui_weak = ui.as_weak();
        ui.on_undo(move || {
            let ui = ui_weak.upgrade().unwrap();
            let mut st = state.borrow_mut();
            if let Some(snap) = st.undo_stack.pop() {
                st.canvas.data = snap;
            }
            let mut b = buffer.borrow_mut();
            copy_to_buffer(&st.canvas, &mut b);
            ui.set_canvas_image(SImage::from_rgb8(b.clone()));
        });
    }
    {
        let state = state.clone();
        let buffer = buffer.clone();
        let ui_weak = ui.as_weak();
        ui.on_clear(move || {
            let ui = ui_weak.upgrade().unwrap();
            let mut st = state.borrow_mut();
            st.snapshot();
            st.canvas = Image::new(W, H, Rgba::WHITE);
            st.brush = None;
            let mut b = buffer.borrow_mut();
            copy_to_buffer(&st.canvas, &mut b);
            ui.set_canvas_image(SImage::from_rgb8(b.clone()));
        });
    }

    // timer de estatísticas
    {
        let ui_weak = ui.as_weak();
        let state = state.clone();
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(500),
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let mut st = state.borrow_mut();
                    let avg = if st.line_n > 0 { st.line_ms_sum / st.line_n as f32 } else { 0.0 };
                    ui.set_stats(format!(
                        "engine: media {:.2} ms/linha | pior {:.2} ms | n={}
(meta < 8 ms em 120 Hz; undo: {} niveis)",
                        avg, st.line_ms_worst, st.line_n, st.undo_stack.len()
                    ).into());
                    st.line_ms_sum = 0.0;
                    st.line_ms_worst = 0.0;
                    st.line_n = 0;
                }
            },
        );
        std::mem::forget(timer);
    }

    ui.run()
}
