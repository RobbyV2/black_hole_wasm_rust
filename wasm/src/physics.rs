use glam::{Vec3, Vec4};

pub const C: f64 = 299792458.0;
pub const G: f64 = 6.67430e-11;

#[derive(Debug, Clone, Copy)]
pub struct BlackHole {
    pub position: Vec3,
    pub mass: f64,
    pub r_s: f64,
}

impl BlackHole {
    pub fn new(position: Vec3, mass: f64) -> Self {
        let r_s = 2.0 * G * mass / (C * C);
        BlackHole {
            position,
            mass,
            r_s,
        }
    }

    pub fn sagittarius_a() -> Self {
        Self::new(Vec3::ZERO, 8.54e36)
    }

    pub fn intercept(&self, px: f32, py: f32, pz: f32) -> bool {
        let dx = px as f64 - self.position.x as f64;
        let dy = py as f64 - self.position.y as f64;
        let dz = pz as f64 - self.position.z as f64;
        let dist2 = dx * dx + dy * dy + dz * dz;
        dist2 < self.r_s * self.r_s
    }

    pub fn schwarzschild_f(&self, r: f64) -> f64 {
        1.0 - self.r_s / r
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectData {
    pub pos_radius: Vec4,
    pub color: Vec4,
    pub mass: f32,
    pub velocity: Vec3,
}

impl ObjectData {
    pub fn new(x: f32, y: f32, z: f32, radius: f32, r: f32, g: f32, b: f32, mass: f32) -> Self {
        ObjectData {
            pos_radius: Vec4::new(x, y, z, radius),
            color: Vec4::new(r, g, b, 1.0),
            mass,
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub r: f64,
    pub theta: f64,
    pub phi: f64,
    pub dr: f64,
    pub dtheta: f64,
    pub dphi: f64,
    pub dt: f64,
    pub energy: f64,
    pub angular_momentum: f64,
}

impl Ray {
    pub fn new(r: f64, theta: f64, phi: f64, dr: f64, dtheta: f64, dphi: f64) -> Self {
        Ray {
            r,
            theta,
            phi,
            dr,
            dtheta,
            dphi,
            dt: 1.0,
            energy: 1.0,
            angular_momentum: 0.0,
        }
    }

    pub fn to_cartesian(&self) -> Vec3 {
        let x = (self.r * self.theta.sin() * self.phi.cos()) as f32;
        let y = (self.r * self.theta.cos()) as f32;
        let z = (self.r * self.theta.sin() * self.phi.sin()) as f32;
        Vec3::new(x, y, z)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Disk {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub thickness: f32,
}

impl Disk {
    pub fn new(inner_radius: f32, outer_radius: f32, thickness: f32) -> Self {
        Disk {
            inner_radius,
            outer_radius,
            thickness,
        }
    }

    pub fn default_accretion_disk() -> Self {
        let r_s = 1.269e10;
        Self::new(r_s * 2.2, r_s * 5.2, 1.0e9)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Planet {
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    pub semi_major_axis: f32,
    pub eccentricity: f32,
    pub mean_motion: f32,
}

impl Planet {
    pub fn new_elliptical_orbit(
        semi_major_axis_scu: f32,
        eccentricity: f32,
        radius: f32,
        black_hole_mass: f64,
    ) -> Self {
        let r_s = 2.0 * G * black_hole_mass / (C * C);
        let unit_scale = r_s as f32 / 2.0;

        let semi_major_axis = semi_major_axis_scu * unit_scale;

        let mean_motion =
            ((G * black_hole_mass / (semi_major_axis as f64).powi(3)).sqrt()) as f32 * 1000.0;

        let position = Vec3::new(semi_major_axis * (1.0 - eccentricity), 0.0, 0.0);

        Planet {
            position,
            velocity: Vec3::ZERO,
            radius: radius * unit_scale,
            semi_major_axis,
            eccentricity,
            mean_motion,
        }
    }

    pub fn update(&mut self, time: f32) {
        let mean_anomaly = self.mean_motion * time;

        let mut eccentric_anomaly = mean_anomaly;
        for _ in 0..10 {
            eccentric_anomaly = mean_anomaly + self.eccentricity * eccentric_anomaly.sin();
        }

        let cos_e = eccentric_anomaly.cos();
        let sin_e = eccentric_anomaly.sin();

        let r = self.semi_major_axis * (1.0 - self.eccentricity * cos_e);

        let x_orbit = self.semi_major_axis * (cos_e - self.eccentricity);
        let z_orbit =
            self.semi_major_axis * (1.0 - self.eccentricity * self.eccentricity).sqrt() * sin_e;

        let inclination = 30.0f32.to_radians();

        self.position.x = x_orbit;
        self.position.y = z_orbit * inclination.sin();
        self.position.z = z_orbit * inclination.cos();

        let vx_orbit =
            -self.semi_major_axis * self.mean_motion * sin_e / (1.0 - self.eccentricity * cos_e);
        let vz_orbit = self.semi_major_axis
            * self.mean_motion
            * (1.0 - self.eccentricity * self.eccentricity).sqrt()
            * cos_e
            / (1.0 - self.eccentricity * cos_e);

        self.velocity.x = vx_orbit;
        self.velocity.y = vz_orbit * inclination.sin();
        self.velocity.z = vz_orbit * inclination.cos();
    }
}
