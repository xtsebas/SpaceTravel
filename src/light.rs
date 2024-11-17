use nalgebra_glm::Vec3;
use crate::color::Color;

pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }
}

impl Light {
    pub fn new_sun() -> Self {
        Light {
            position: Vec3::new(0.0, 0.0, 0.0), // Posición en el centro del sistema
            color: Color::new(255, 229, 179),   // Color cálido del Sol en formato RGB
            intensity: 1.5,                     // Intensidad alta para simular la luz solar
        }
    }
}
