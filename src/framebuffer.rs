// framebuffer.rs
use crate::Vec3;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub zbuffer: Vec<f32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            zbuffer: vec![f32::INFINITY; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
        for depth in self.zbuffer.iter_mut() {
            *depth = f32::INFINITY;
        }
    }

    pub fn point(&mut self, x: usize, y: usize, depth: f32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            if self.zbuffer[index] > depth {
                self.buffer[index] = self.current_color;
                self.zbuffer[index] = depth;
            }
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
}

impl Framebuffer {
    pub fn draw_triangle(
        &mut self,
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        texture: &image::DynamicImage,
    ) {
        // Convertir las coordenadas homogéneas a coordenadas de pantalla
        let to_screen = |v: Vec3| {
            let x = ((v.x + 1.0) * 0.5 * self.width as f32) as usize;
            let y = ((1.0 - (v.y + 1.0) * 0.5) * self.height as f32) as usize;
            (x, y)
        };

        let screen_v0 = to_screen(v0);
        let screen_v1 = to_screen(v1);
        let screen_v2 = to_screen(v2);

        // Dibujar los bordes del triángulo (opcional, puedes implementar un relleno si es necesario)
        self.draw_line(screen_v0.0, screen_v0.1, screen_v1.0, screen_v1.1, self.current_color);
        self.draw_line(screen_v1.0, screen_v1.1, screen_v2.0, screen_v2.1, self.current_color);
        self.draw_line(screen_v2.0, screen_v2.1, screen_v0.0, screen_v0.1, self.current_color);
    }

    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        // Implementación del algoritmo de Bresenham para líneas
        let mut x0 = x0 as isize;
        let mut y0 = y0 as isize;
        let x1 = x1 as isize;
        let y1 = y1 as isize;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let mut sx = if x0 < x1 { 1 } else { -1 };
        let mut sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        while x0 != x1 || y0 != y1 {
            if x0 >= 0 && x0 < self.width as isize && y0 >= 0 && y0 < self.height as isize {
                self.buffer[y0 as usize * self.width + x0 as usize] = color;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                }
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                }
                err += dx;
                y0 += sy;
            }
        }
    }
}
