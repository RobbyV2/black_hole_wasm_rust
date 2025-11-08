use glam::{Mat4, Vec3};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Camera {
    pub target: Vec3,
    pub radius: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub orbit_speed: f32,
    pub zoom_speed: f32,
    pub dragging: bool,
    pub moving: bool,
    pub last_x: f64,
    pub last_y: f64,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            target: Vec3::ZERO,
            radius: 1.67e11,
            min_radius: 1e10,
            max_radius: 1e12,
            azimuth: 0.0,
            elevation: 1.66,
            orbit_speed: 0.01,
            zoom_speed: 25e9,
            dragging: false,
            moving: false,
            last_x: 0.0,
            last_y: 0.0,
        }
    }

    pub fn position(&self) -> Vec3 {
        let clamped_elevation = self.elevation.clamp(0.01, PI - 0.01);
        Vec3::new(
            self.radius * clamped_elevation.sin() * self.azimuth.cos(),
            self.radius * clamped_elevation.cos(),
            self.radius * clamped_elevation.sin() * self.azimuth.sin(),
        )
    }

    pub fn update(&mut self) {
        self.target = Vec3::ZERO;
        self.moving = self.dragging;
    }

    pub fn process_mouse_move(&mut self, x: f64, y: f64) {
        let dx = (x - self.last_x) as f32;
        let dy = (y - self.last_y) as f32;

        if self.dragging {
            self.azimuth += dx * self.orbit_speed;
            self.elevation -= dy * self.orbit_speed;
            self.elevation = self.elevation.clamp(0.01, PI - 0.01);
        }

        self.last_x = x;
        self.last_y = y;
        self.update();
    }

    pub fn process_mouse_button(&mut self, button: u8, pressed: bool, x: f64, y: f64) {
        if button == 0 {
            if pressed {
                self.dragging = true;
                self.last_x = x;
                self.last_y = y;
            } else {
                self.dragging = false;
            }
        }
    }

    pub fn process_scroll(&mut self, yoffset: f64) {
        self.radius -= yoffset as f32 * self.zoom_speed;
        self.radius = self.radius.clamp(self.min_radius, self.max_radius);
        self.update();
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }

    pub fn projection_matrix(&self, aspect: f32, fov: f32) -> Mat4 {
        Mat4::perspective_rh(fov.to_radians(), aspect, 1e8, 1e13)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
