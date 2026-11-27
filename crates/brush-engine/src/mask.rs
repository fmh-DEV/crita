//! Máscara do "auto brush" — portado de libs/brush/KisAutoBrush (Krita):
//! círculo com falloff de hardness e borda anti-aliased de ~1px.
//!
//! valor(d) = 1                              se d <= raio·hardness
//!          = 0                              se d >= raio
//!          = 1 − smoothstep(t), t=(d−inner)/(raio−inner)  caso contrário

pub struct AutoBrushMask {
    pub radius: f32,
    pub hardness: f32,
    pub size: usize, // lado da grade (ímpar)
    data: Vec<f32>,
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

impl AutoBrushMask {
    /// Gera a máscara para raio e hardness (0..1). Cachear por (raio, hardness)
    /// — o Krita faz o mesmo com KisBrush.
    pub fn new(radius: f32, hardness: f32) -> AutoBrushMask {
        assert!(radius > 0.0, "raio deve ser positivo");
        let hardness = hardness.clamp(0.0, 1.0);
        let size = (radius.ceil() as usize) * 2 + 1;
        let mut data = vec![0.0f32; size * size];
        let inner = radius * hardness;
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - radius;
                let dy = y as f32 - radius;
                let d = (dx * dx + dy * dy).sqrt();
                data[y * size + x] = if d <= inner {
                    1.0
                } else if d >= radius {
                    0.0
                } else {
                    let t = ((d - inner) / (radius - inner).max(1e-6)).clamp(0.0, 1.0);
                    1.0 - smoothstep(t)
                };
            }
        }
        AutoBrushMask { radius, hardness, size, data }
    }

    /// Intensidade (0..1) da máscara na célula (x, y) da grade.
    pub fn at(&self, x: usize, y: usize) -> f32 {
        self.data[y * self.size + x]
    }
}
