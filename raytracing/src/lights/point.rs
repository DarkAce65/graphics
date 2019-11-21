use super::LightColor;
use crate::core::Object3D;
use derive_builder::Builder;
use nalgebra::{Point3, Vector3};

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct PointLight {
    position: Point3<f64>,
    color: Vector3<f64>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            color: Vector3::from([1.0; 3]),
        }
    }
}

impl Object3D for PointLight {
    fn position(&self) -> Point3<f64> {
        self.position
    }
}

impl LightColor for PointLight {
    fn get_color(&self) -> Vector3<f64> {
        self.color
    }
}