use crate::physics::{C, G, Ray};
use glam::Vec3;

const SAG_A_MASS: f64 = 8.54e36;
const SAG_A_RS: f64 = 2.0 * G * SAG_A_MASS / (C * C);
const D_LAMBDA: f64 = 1e7;
const ESCAPE_R: f64 = 1e30;

pub fn init_ray(pos: Vec3, dir: Vec3) -> Ray {
    let r = pos.length() as f64;
    let theta = (pos.z as f64 / r).acos();
    let phi = (pos.y as f64).atan2(pos.x as f64);

    let dx = dir.x as f64;
    let dy = dir.y as f64;
    let dz = dir.z as f64;

    let dr = theta.sin() * phi.cos() * dx + theta.sin() * phi.sin() * dy + theta.cos() * dz;

    let dtheta =
        (theta.cos() * phi.cos() * dx + theta.cos() * phi.sin() * dy - theta.sin() * dz) / r;

    let dphi = (-phi.sin() * dx + phi.cos() * dy) / (r * theta.sin());

    let angular_momentum = r * r * theta.sin() * dphi;
    let f = 1.0 - SAG_A_RS / r;
    let dt_dl = ((dr * dr / f)
        + r * r * (dtheta * dtheta + theta.sin() * theta.sin() * dphi * dphi))
        .sqrt();
    let energy = f * dt_dl;

    Ray {
        r,
        theta,
        phi,
        dr,
        dtheta,
        dphi,
        dt: dt_dl,
        energy,
        angular_momentum,
    }
}

fn geodesic_rhs(ray: &Ray, r_s: f64) -> (Vec3, Vec3) {
    let r = ray.r;
    let theta = ray.theta;
    let dr = ray.dr;
    let dtheta = ray.dtheta;
    let dphi = ray.dphi;

    let f = 1.0 - r_s / r;
    let dt_dl = ray.energy / f;

    let d1 = Vec3::new(dr as f32, dtheta as f32, dphi as f32);

    let d2r = -(r_s / (2.0 * r * r)) * f * dt_dl * dt_dl
        + (r_s / (2.0 * r * r * f)) * dr * dr
        + r * (dtheta * dtheta + theta.sin() * theta.sin() * dphi * dphi);

    let d2theta = -2.0 * dr * dtheta / r + theta.sin() * theta.cos() * dphi * dphi;

    let d2phi = -2.0 * dr * dphi / r - 2.0 * theta.cos() / theta.sin() * dtheta * dphi;

    let d2 = Vec3::new(d2r as f32, d2theta as f32, d2phi as f32);

    (d1, d2)
}

pub fn rk4_step(ray: &mut Ray, dl: f64, r_s: f64) {
    let (k1a, k1b) = geodesic_rhs(ray, r_s);

    ray.r += dl * k1a.x as f64;
    ray.theta += dl * k1a.y as f64;
    ray.phi += dl * k1a.z as f64;
    ray.dr += dl * k1b.x as f64;
    ray.dtheta += dl * k1b.y as f64;
    ray.dphi += dl * k1b.z as f64;
}

pub fn trace_ray(pos: Vec3, dir: Vec3, r_s: f64, max_steps: usize) -> TraceResult {
    let mut ray = init_ray(pos, dir);

    for _ in 0..max_steps {
        if ray.r <= r_s {
            return TraceResult::HitBlackHole;
        }

        rk4_step(&mut ray, D_LAMBDA, r_s);

        if ray.r > ESCAPE_R {
            return TraceResult::Escaped;
        }
    }

    TraceResult::MaxSteps
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraceResult {
    HitBlackHole,
    HitDisk,
    HitObject,
    Escaped,
    MaxSteps,
}
