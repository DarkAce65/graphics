mod raytracing_scene;
mod scene;

use crate::utils;
use nalgebra::{clamp, Point3, Unit, Vector3};
use num_traits::Zero;
use serde::Deserialize;
use std::ops::AddAssign;

pub use scene::Scene;

const GAMMA: f64 = 2.2;
const BIAS: f64 = 1e-10;

pub struct ColorData {
    color: Vector3<f64>,
    albedo: Vector3<f64>,
    emissive: Vector3<f64>,
    ambient_occlusion: f64,
}

impl ColorData {
    fn new(color: Vector3<f64>, albedo: Vector3<f64>, emissive: Vector3<f64>) -> Self {
        Self {
            color,
            albedo,
            emissive,
            ambient_occlusion: 1.0,
        }
    }

    fn zero() -> Self {
        Self {
            color: Vector3::zero(),
            albedo: Vector3::zero(),
            emissive: Vector3::zero(),
            ambient_occlusion: 0.0,
        }
    }

    fn black() -> Self {
        Self::new(Vector3::zero(), Vector3::zero(), Vector3::zero())
    }

    fn clamp(mut self) -> Self {
        self.color = self.color.map(|c| clamp(c, 0.0, 1.0));
        self.albedo = self.albedo.map(|c| clamp(c, 0.0, 1.0));
        self.emissive = self.emissive.map(|c| clamp(c, 0.0, 1.0));
        self.ambient_occlusion = clamp(self.ambient_occlusion, 0.0, 1.0);
        self
    }

    fn gamma_correct(mut self) -> Self {
        self.color = utils::gamma_correct(self.color, GAMMA);
        self
    }
}

#[derive(Copy, Clone)]
pub struct CastStats {
    pub ray_count: u64,
}

impl CastStats {
    pub const fn zero() -> Self {
        Self { ray_count: 0 }
    }
}

impl AddAssign for CastStats {
    fn add_assign(&mut self, rhs: Self) {
        self.ray_count += rhs.ray_count;
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Camera {
    pub fov: f64,
    pub position: Point3<f64>,
    pub target: Point3<f64>,
    pub up: Unit<Vector3<f64>>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 65.0,
            position: Point3::from([0.0, 0.0, 1.0]),
            target: Point3::origin(),
            up: Vector3::y_axis(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub max_depth: u8,
    pub samples_per_pixel: u16,
    pub max_reflected_rays: u16,
    pub max_occlusion_rays: u16,
    pub max_occlusion_distance: f64,
    pub occlusion_blur_radius: u16,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            max_depth: 3,
            samples_per_pixel: 4,
            max_reflected_rays: 32,
            max_occlusion_rays: 16,
            max_occlusion_distance: 1.0,
            occlusion_blur_radius: 2,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::{Material, PhongMaterial, Transform};
    use crate::lights::{AmbientLight, Light, PointLight};
    use crate::primitives::{Cube, Object3D};
    use serde_json::json;

    #[test]
    fn it_builds_a_raytracing_scene_from_an_empty_scene_json() {
        let scene_json = json!({});
        let scene: Result<Scene, serde_json::error::Error> = serde_json::from_value(scene_json);
        assert!(scene.is_ok(), "failed to deserialize scene");

        scene.unwrap().build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_a_scene_json() {
        let scene_json = json!({
          "max_depth": 5,
          "width": 200,
          "height": 200,
          "camera": { "position": [2, 5, 15], "target": [-1, 0, 0] },
          "lights": [
            { "type": "ambient", "color": [0.01, 0.01, 0.01] },
            {
              "type": "point",
              "transform": [{ "translate": [-8, 3, 0] }],
              "color": [0.5, 0.5, 0.5]
            }
          ],
          "objects": [
            {
              "type": "cube",
              "size": 1,
              "transform": [{ "rotate": [[0, 1, 0], 30] }, { "translate": [0, 2, 0] }],
              "material": { "type": "phong", "color": [1, 0.1, 0.1] }
            }
          ]
        });

        let scene: Result<Scene, serde_json::error::Error> = serde_json::from_value(scene_json);
        assert!(scene.is_ok(), "failed to deserialize scene");

        scene.unwrap().build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_an_empty_scene() {
        let scene = Scene::new(RenderOptions::default(), Camera::default());
        scene.build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_a_scene() {
        let mut scene = Scene::new(
            RenderOptions {
                width: 200,
                height: 200,
                max_depth: 5,
                ..RenderOptions::default()
            },
            Camera::default(),
        );

        scene.add_light(Light::Ambient(AmbientLight::new(Vector3::from([
            0.01, 0.01, 0.01,
        ]))));
        scene.add_light(Light::Point(Box::new(PointLight::new(
            Vector3::from([0.5, 0.5, 0.5]),
            1.0,
            Transform::identity().translate(Vector3::from([-8.0, 3.0, 0.0])),
        ))));

        scene.add_object(Object3D::Cube(Box::new(Cube::new(
            1.0,
            Transform::identity()
                .rotate(Vector3::y_axis(), 30.0)
                .translate(Vector3::from([0.0, 2.0, 0.0])),
            Material::Phong(PhongMaterial {
                color: Vector3::from([1.0, 0.1, 0.1]),
                ..PhongMaterial::default()
            }),
        ))));

        scene.build_raytracing_scene();
    }
}
