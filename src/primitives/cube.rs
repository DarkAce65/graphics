use super::{HasMaterial, Object3D, Primitive, RaytracingObject};
use crate::core::{
    BoundingVolume, Material, MaterialSide, ObjectWithBounds, Transform, Transformed,
};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray, RayType};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Cube {
    size: f64,
    transform: Transform,
    pub material: Material,

    pub children: Option<Vec<Object3D>>,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            size: 1.0,
            transform: Transform::default(),
            material: Material::default(),

            children: None,
        }
    }
}

impl Cube {
    pub fn new(size: f64, transform: Transform, material: Material) -> Self {
        Self {
            size,
            transform,
            material,
            ..Cube::default()
        }
    }

    pub fn add_child(&mut self, object: Object3D) {
        if let Some(children) = self.children.as_mut() {
            children.push(object);
        }
    }

    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn RaytracingObject>> {
        let transform = transform * self.transform;

        let mut objects: Vec<Box<dyn RaytracingObject>> = Vec::new();

        if let Some(children) = self.children {
            for child in children {
                let child_objects: Vec<Box<dyn RaytracingObject>> =
                    child.flatten_to_world(&transform);
                objects.extend(child_objects);
            }
        }

        objects.push(Box::new(RaytracingCube::new(
            self.size,
            transform,
            self.material,
        )));

        objects
    }
}

#[derive(Debug)]
pub struct RaytracingCube {
    size: f64,
    world_transform: Transform,
    material: Material,
}

impl RaytracingCube {
    pub fn new(size: f64, world_transform: Transform, material: Material) -> Self {
        Self {
            size,
            world_transform,
            material,
        }
    }
}

impl HasMaterial for RaytracingCube {
    fn get_material(&self) -> &Material {
        &self.material
    }
}

impl Transformed for RaytracingCube {
    fn get_transform(&self) -> &Transform {
        &self.world_transform
    }
}

impl Intersectable for RaytracingCube {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let ray_sign = ray.direction.map(|c| c.signum());
        let half = self.size / 2.0;

        let d0 = (-ray.origin.x - ray_sign.x * half) / ray.direction.x;
        let d1 = (-ray.origin.x + ray_sign.x * half) / ray.direction.x;
        let dy_min = (-ray.origin.y - ray_sign.y * half) / ray.direction.y;
        let dy_max = (-ray.origin.y + ray_sign.y * half) / ray.direction.y;

        if dy_max < d0 || d1 < dy_min {
            return None;
        }

        let d0 = if dy_min > d0 { dy_min } else { d0 };
        let d1 = if d1 > dy_max { dy_max } else { d1 };

        let dz_min = (-ray.origin.z - ray_sign.z * half) / ray.direction.z;
        let dz_max = (-ray.origin.z + ray_sign.z * half) / ray.direction.z;

        if dz_max < d0 || d1 < dz_min {
            return None;
        }

        let d0 = if dz_min > d0 { dz_min } else { d0 };
        let d1 = if d1 > dz_max { dz_max } else { d1 };

        debug_assert!(d0 <= d1);

        let d = match (self.material.side(), ray.ray_type) {
            (MaterialSide::Both, _) | (_, RayType::Shadow) => {
                if d0 < 0.0 {
                    d1
                } else {
                    d0
                }
            }
            (MaterialSide::Front, _) => d0,
            (MaterialSide::Back, _) => d1,
        };
        if d < 0.0 {
            return None;
        }

        Some(Intersection::new(self, d))
    }
}

impl Primitive for RaytracingCube {
    fn into_bounded_object(self: Box<Self>) -> ObjectWithBounds {
        let half = self.size / 2.0;
        let bounding_volume = BoundingVolume::from_bounds_and_transform(
            Point3::from([-half; 3]),
            Point3::from([half; 3]),
            self.get_transform(),
        );

        ObjectWithBounds::bounded(self, bounding_volume)
    }

    fn surface_normal(
        &self,
        object_hit_point: &Point3<f64>,
        _intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        let normal = object_hit_point.coords;
        let normal_sign = normal.map(|c| c.signum());
        let normal = normal.map(|c| c.abs());
        if normal.x > normal.y {
            if normal.x > normal.z {
                if normal_sign.x < 0.0 {
                    -Vector3::x_axis()
                } else {
                    Vector3::x_axis()
                }
            } else if normal_sign.z < 0.0 {
                -Vector3::z_axis()
            } else {
                Vector3::z_axis()
            }
        } else if normal.y > normal.z {
            if normal_sign.y < 0.0 {
                -Vector3::y_axis()
            } else {
                Vector3::y_axis()
            }
        } else if normal_sign.z < 0.0 {
            -Vector3::z_axis()
        } else {
            Vector3::z_axis()
        }
    }

    fn uv(
        &self,
        object_hit_point: &Point3<f64>,
        object_normal: &Unit<Vector3<f64>>,
        _intermediate: IntermediateData,
    ) -> Vector2<f64> {
        let hit_point = object_hit_point.coords.map(|c| c / self.size);

        if object_normal.x > object_normal.y {
            if object_normal.x > object_normal.z {
                Vector2::new(hit_point.y + 0.5, hit_point.z + 0.5)
            } else {
                Vector2::new(hit_point.x + 0.5, hit_point.y + 0.5)
            }
        } else if object_normal.y > object_normal.z {
            Vector2::new(hit_point.x + 0.5, hit_point.z + 0.5)
        } else {
            Vector2::new(hit_point.x + 0.5, hit_point.y + 0.5)
        }
    }
}
