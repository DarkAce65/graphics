use super::{HasMaterial, Intersectable, Loadable, Primitive};
use crate::core::{BoundingVolume, Bounds, Material, MaterialSide, Transform, Transformed};
use crate::ray_intersection::{IntermediateData, Intersection, Ray, RayType};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use num_traits::identities::Zero;
use serde::Deserialize;
use std::f64::EPSILON;

#[derive(Debug)]
struct VertexPNT {
    position: Point3<f64>,
    normal: Unit<Vector3<f64>>,
    texcoords: Vector2<f64>,
}

impl VertexPNT {
    fn new(position: Point3<f64>, normal: Unit<Vector3<f64>>, texcoords: Vector2<f64>) -> Self {
        Self {
            position,
            normal,
            texcoords,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TriangleData {
    #[serde(default)]
    transform: Transform,
    vertices: [Point3<f64>; 3],
    material: Material,
    children: Option<Vec<Box<dyn Primitive>>>,
}

impl Default for TriangleData {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            vertices: [Point3::origin(), Point3::origin(), Point3::origin()],
            material: Material::default(),
            children: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "TriangleData")]
pub struct Triangle {
    transform: Transform,
    vertex_data: [VertexPNT; 3],
    material: Material,
    children: Option<Vec<Box<dyn Primitive>>>,
}

impl From<TriangleData> for Triangle {
    fn from(data: TriangleData) -> Self {
        let normals = [Triangle::compute_normal(data.vertices); 3];
        let texcoords = [Vector2::zero(); 3];

        Triangle::new(
            data.transform,
            data.vertices,
            normals,
            texcoords,
            data.material,
            data.children,
        )
    }
}

impl Triangle {
    pub fn new(
        transform: Transform,
        positions: [Point3<f64>; 3],
        normals: [Unit<Vector3<f64>>; 3],
        texcoords: [Vector2<f64>; 3],
        material: Material,
        children: Option<Vec<Box<dyn Primitive>>>,
    ) -> Self {
        let vertex_data = [
            VertexPNT::new(positions[0], normals[0], texcoords[0]),
            VertexPNT::new(positions[1], normals[1], texcoords[1]),
            VertexPNT::new(positions[2], normals[2], texcoords[2]),
        ];

        Self {
            transform,
            vertex_data,
            material,
            children,
        }
    }

    pub fn compute_normal(vertices: [Point3<f64>; 3]) -> Unit<Vector3<f64>> {
        let edge1 = vertices[1] - vertices[0];
        let edge2 = vertices[2] - vertices[0];

        Unit::new_normalize(edge1.cross(&edge2))
    }
}

impl HasMaterial for Triangle {
    fn get_material(&self) -> &Material {
        &self.material
    }

    fn get_material_mut(&mut self) -> &mut Material {
        &mut self.material
    }
}

impl Loadable for Triangle {}

impl Transformed for Triangle {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}

impl Intersectable for Triangle {
    fn make_bounding_volume(&self, transform: &Transform) -> Bounds {
        let mut min = self.vertex_data[0].position;
        let mut max = min;
        for VertexPNT { position, .. } in self.vertex_data[1..].iter() {
            min.x = min.x.min(position.x);
            min.y = min.y.min(position.y);
            min.z = min.z.min(position.z);

            max.x = max.x.max(position.x);
            max.y = max.y.max(position.y);
            max.z = max.z.max(position.z);
        }

        Bounds::Bounded(BoundingVolume::from_bounds_and_transform(
            min, max, transform,
        ))
    }

    fn get_children(&self) -> Option<&Vec<Box<dyn Primitive>>> {
        self.children.as_ref()
    }

    fn get_children_mut(&mut self) -> Option<&mut Vec<Box<dyn Primitive>>> {
        self.children.as_mut()
    }

    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let edge1 = self.vertex_data[1].position - self.vertex_data[0].position;
        let edge2 = self.vertex_data[2].position - self.vertex_data[0].position;
        let p_vec = ray.direction.cross(&edge2);
        let det = edge1.dot(&p_vec);

        if match (self.material.side(), ray.ray_type) {
            (MaterialSide::Both, _) | (_, RayType::Shadow) => det.abs() < EPSILON,
            (MaterialSide::Front, _) => det < EPSILON,
            (MaterialSide::Back, _) => -det < EPSILON,
        } {
            return None;
        }

        let t_vec = ray.origin - self.vertex_data[0].position;
        let u = t_vec.dot(&p_vec) / det;
        if u < 0.0 || 1.0 < u {
            return None;
        }

        let q_vec = t_vec.cross(&edge1);
        let v = ray.direction.dot(&q_vec) / det;
        if v < 0.0 || 1.0 < u + v {
            return None;
        }

        let distance = edge2.dot(&q_vec) / det;

        Some(Intersection::new_with_data(
            self,
            distance,
            IntermediateData::Barycentric(u, v, 1.0 - u - v),
        ))
    }

    fn surface_normal(
        &self,
        _object_hit_point: &Point3<f64>,
        intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        let (u, v, w) = match intermediate {
            IntermediateData::Barycentric(u, v, w) => (u, v, w),
            _ => unreachable!(),
        };

        Unit::new_normalize(
            w * self.vertex_data[0].normal.into_inner()
                + u * self.vertex_data[1].normal.into_inner()
                + v * self.vertex_data[2].normal.into_inner(),
        )
    }

    fn uv(
        &self,
        _object_hit_point: &Point3<f64>,
        _object_normal: &Unit<Vector3<f64>>,
        intermediate: IntermediateData,
    ) -> Vector2<f64> {
        let (u, v, w) = match intermediate {
            IntermediateData::Barycentric(u, v, w) => (u, v, w),
            _ => unreachable!(),
        };

        w * self.vertex_data[0].texcoords
            + u * self.vertex_data[1].texcoords
            + v * self.vertex_data[2].texcoords
    }
}
