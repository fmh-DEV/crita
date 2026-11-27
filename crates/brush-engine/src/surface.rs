//! Superfície de pixels e cor RGBA (straight alpha, 0..1).

/// Cor em RGBA com alfa "straight" (não premultiplicado), canais em 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: Rgba = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Rgba = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    pub fn rgb(r: f32, g: f32, b: f32) -> Rgba {
        Rgba { r, g, b, a: 1.0 }
    }

    /// Constrói a partir de hex "#rrggbb" (formato do seletor de cor da UI).
    pub fn from_hex(hex: &str) -> Rgba {
        let h = hex.trim_start_matches('#');
        let v = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0) as f32 / 255.0;
        if h.len() >= 6 {
            Rgba::rgb(v(&h[0..2]), v(&h[2..4]), v(&h[4..6]))
        } else {
            Rgba::BLACK
        }
    }
}

/// Abstração mínima de canvas. Substitui o KisPaintDevice do Krita.
pub trait Surface {
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn get(&self, x: usize, y: usize) -> Rgba;

    /// Escreve o pixel diretamente (sem composição).
    fn set_pixel(&mut self, x: usize, y: usize, c: Rgba);

    /// Compõe `src` sobre o pixel atual (src-over; ver `blend::src_over`).
    fn blend_pixel(&mut self, x: usize, y: usize, src: Rgba) {
        let dst = self.get(x, y);
        self.set_pixel(x, y, crate::blend::src_over(dst, src));
    }

    /// Média ponderada (pela máscara do brush) dos pixels sob o círculo.
    /// Usado pelo color smudge. Base: caixa delimitadora com pesos da máscara
    /// (o Krita amostra de forma mais elaborada; o modelo é o mesmo).
    fn average_color(&self, cx: f32, cy: f32, radius: f32) -> Rgba {
        let x0 = (cx - radius).floor().max(0.0) as usize;
        let y0 = (cy - radius).floor().max(0.0) as usize;
        let x1 = (cx + radius).ceil().min(self.width() as f32) as usize;
        let y1 = (cy + radius).ceil().min(self.height() as f32) as usize;
        if x1 <= x0 || y1 <= y0 {
            return Rgba::TRANSPARENT;
        }
        let (mut r, mut g, mut b, mut ws) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for y in y0..y1 {
            for x in x0..x1 {
                // peso aproximado em disco: ignora cantos fora do círculo
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                if dx * dx + dy * dy > radius * radius {
                    continue;
                }
                let c = self.get(x, y);
                let a = c.a;
                r += c.r * a;
                g += c.g * a;
                b += c.b * a;
                ws += a + (1.0 - a) * 0.15;
            }
        }
        if ws <= 1e-6 {
            return Rgba::TRANSPARENT;
        }
        Rgba { r: r / ws, g: g / ws, b: b / ws, a: 1.0 }
    }
}

/// Surface em memória contígua (linha-major). Equivalente simples de QImage.
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Rgba>,
}

impl Image {
    pub fn new(width: usize, height: usize, fill: Rgba) -> Image {
        Image { width, height, data: vec![fill; width * height] }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

impl Surface for Image {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn get(&self, x: usize, y: usize) -> Rgba {
        self.data[self.idx(x, y)]
    }
    fn set_pixel(&mut self, x: usize, y: usize, c: Rgba) {
        let i = self.idx(x, y);
        self.data[i] = c;
    }
}
