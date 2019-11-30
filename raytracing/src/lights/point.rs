use super::LightColor;
use crate::core::{Object3D, Transform};
use derive_builder::Builder;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Builder, Copy, Clone, Debug, Serialize, Deserialize)]
#[builder(default)]
pub struct PointLight {
    transform: Transform,
    color: Vector3<f64>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            color: Vector3::from([1.0; 3]),
        }
    }
}

impl Object3D for PointLight {
    fn transform(&self) -> Transform {
        self.transform
    }
}

impl LightColor for PointLight {
    fn get_color(&self) -> Vector3<f64> {
        self.color
    }
}
