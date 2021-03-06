use auto_ops::impl_op_ex;
use nalgebra::{Affine3, Matrix4, Point3, Rotation3, Translation3, Unit, Vector3};
use once_cell::sync::OnceCell;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::default::Default;
use std::fmt;

pub trait Transformed {
    fn get_transform(&self) -> &Transform;
    fn get_position(&self) -> Point3<f64> {
        self.get_transform().matrix() * Point3::origin()
    }
}

#[derive(Clone, PartialEq)]
pub struct Transform {
    matrix: Affine3<f64>,
    inv_matrix: OnceCell<Affine3<f64>>,
    inv_transpose_matrix: OnceCell<Affine3<f64>>,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl fmt::Debug for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transform {{ matrix: {:?} }}", self.matrix)
    }
}

impl_op_ex!(*|a: &Transform, b: &Transform| -> Transform { Transform::new(a.matrix * b.matrix) });

impl Transform {
    pub fn identity() -> Self {
        Self::new(Affine3::identity())
    }

    pub fn new(matrix: Affine3<f64>) -> Self {
        Self {
            matrix,
            inv_matrix: OnceCell::new(),
            inv_transpose_matrix: OnceCell::new(),
        }
    }

    pub fn matrix(&self) -> Affine3<f64> {
        self.matrix
    }

    pub fn inverse(&self) -> Affine3<f64> {
        *self.inv_matrix.get_or_init(|| self.matrix.inverse())
    }

    pub fn inverse_transpose(&self) -> Affine3<f64> {
        *self.inv_transpose_matrix.get_or_init(|| {
            Affine3::from_matrix_unchecked(
                nalgebra::convert::<Affine3<f64>, Matrix4<f64>>(self.inverse()).transpose(),
            )
        })
    }

    fn set_matrix(mut self, m: Affine3<f64>) -> Self {
        self.matrix = m;
        self.inv_matrix = OnceCell::new();
        self.inv_transpose_matrix = OnceCell::new();
        self
    }

    pub fn translate(self, translation: Vector3<f64>) -> Self {
        let translated = Translation3::from(translation) * self.matrix;
        self.set_matrix(translated)
    }

    pub fn rotate(self, axis: Unit<Vector3<f64>>, angle: f64) -> Self {
        let rotated = Rotation3::from_axis_angle(&axis, angle.to_radians()) * self.matrix;
        self.set_matrix(rotated)
    }

    pub fn scale(self, scale: Vector3<f64>) -> Self {
        let scaled =
            Affine3::from_matrix_unchecked(Matrix4::new_nonuniform_scaling(&scale)) * self.matrix;
        self.set_matrix(scaled)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
enum SubTransform {
    Translate(Vector3<f64>),
    Rotate(Unit<Vector3<f64>>, f64),
    Scale(Vector3<f64>),
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransformVisitor;

        impl<'de> Visitor<'de> for TransformVisitor {
            type Value = Transform;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Transform")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Transform, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut transform = Transform::default();
                loop {
                    let next: Option<SubTransform> = seq.next_element()?;
                    if let Some(next) = next {
                        transform = match next {
                            SubTransform::Translate(translation) => {
                                transform.translate(translation)
                            }
                            SubTransform::Rotate(axis, angle) => transform.rotate(axis, angle),
                            SubTransform::Scale(scale) => transform.scale(scale),
                        };
                    } else {
                        break;
                    }
                }

                Ok(transform)
            }
        }

        deserializer.deserialize_seq(TransformVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nalgebra::{Affine3, Matrix4, Vector3};
    use serde_json::json;

    #[test]
    fn it_constructs_matrices() {
        let default = Transform::default();
        let translation = Transform::default().translate(Vector3::from([1.0, 2.0, 3.0]));
        let rotation = Transform::default().rotate(Vector3::y_axis(), 50.0);
        let scale = Transform::default().scale(Vector3::from([1.0, 2.0, 3.0]));

        // Base transform matrix
        assert_eq!(default.matrix(), Affine3::identity());

        assert_eq!(
            translation.matrix(),
            Affine3::from_matrix_unchecked(Matrix4::new_translation(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
        );
        assert_eq!(
            rotation.matrix(),
            Affine3::from_matrix_unchecked(Matrix4::from_axis_angle(
                &Vector3::y_axis(),
                50.0_f64.to_radians()
            ))
        );
        assert_eq!(
            scale.matrix(),
            Affine3::from_matrix_unchecked(Matrix4::new_nonuniform_scaling(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
        );

        // Inverse transform matrix
        assert_eq!(default.inverse(), Affine3::identity().inverse());

        assert_eq!(
            translation.inverse(),
            Affine3::from_matrix_unchecked(Matrix4::new_translation(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
            .inverse()
        );
        assert_eq!(
            rotation.inverse(),
            Affine3::from_matrix_unchecked(Matrix4::from_axis_angle(
                &Vector3::y_axis(),
                50.0_f64.to_radians()
            ))
            .inverse()
        );
        assert_eq!(
            scale.inverse(),
            Affine3::from_matrix_unchecked(Matrix4::new_nonuniform_scaling(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
            .inverse()
        );

        // Inverse transpose transform matrix
        assert_eq!(
            default.inverse_transpose(),
            Affine3::from_matrix_unchecked(Matrix4::identity().transpose()).inverse()
        );

        assert_eq!(
            translation.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::new_translation(&Vector3::from([1.0, 2.0, 3.0])).transpose()
            )
            .inverse()
        );
        assert_eq!(
            rotation.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0_f64.to_radians()).transpose()
            )
            .inverse()
        );
        assert_eq!(
            scale.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::new_nonuniform_scaling(&Vector3::from([1.0, 2.0, 3.0])).transpose()
            )
            .inverse()
        );
    }

    #[test]
    fn it_constructs_complex_matrices() {
        let full = Transform::default()
            .rotate(Vector3::y_axis(), 50.0)
            .scale(Vector3::from([3.0, 2.0, 1.0]))
            .translate(Vector3::from([5.0, 2.0, 3.0]));
        let translation_identity = Transform::default()
            .translate(Vector3::from([1.0, 2.0, 3.0]))
            .translate(Vector3::from([-1.0, -2.0, -3.0]));
        let full_identity = Transform::default()
            .rotate(Vector3::y_axis(), 50.0)
            .scale(Vector3::from([1.0, 2.0, 4.0]))
            .translate(Vector3::from([1.0, 2.0, 3.0]))
            .translate(Vector3::from([-1.0, -2.0, -3.0]))
            .scale(Vector3::from([1.0, 0.5, 0.25]))
            .rotate(Vector3::y_axis(), -50.0);

        // Base transform matrix
        assert_eq!(
            full.matrix(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0_f64.to_radians())
                    .append_nonuniform_scaling(&Vector3::from([3.0, 2.0, 1.0]))
                    .append_translation(&Vector3::from([5.0, 2.0, 3.0]))
            )
        );
        assert_eq!(translation_identity.matrix(), Affine3::identity());
        assert_eq!(full_identity.matrix(), Affine3::identity());

        // Inverse transform matrix
        assert_eq!(
            full.inverse(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0_f64.to_radians())
                    .append_nonuniform_scaling(&Vector3::from([3.0, 2.0, 1.0]))
                    .append_translation(&Vector3::from([5.0, 2.0, 3.0]))
            )
            .inverse()
        );
        assert_eq!(
            translation_identity.inverse(),
            Affine3::identity().inverse()
        );
        assert_eq!(full_identity.inverse(), Affine3::identity().inverse());

        // Inverse transpose transform matrix
        assert_eq!(
            full.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0_f64.to_radians())
                    .append_nonuniform_scaling(&Vector3::from([3.0, 2.0, 1.0]))
                    .append_translation(&Vector3::from([5.0, 2.0, 3.0]))
                    .transpose()
            )
            .inverse()
        );
        assert_eq!(
            translation_identity.inverse_transpose(),
            Affine3::from_matrix_unchecked(Matrix4::identity().transpose()).inverse()
        );
        assert_eq!(
            full_identity.inverse_transpose(),
            Affine3::from_matrix_unchecked(Matrix4::identity().transpose()).inverse()
        );
    }

    #[test]
    fn it_deserializes_identity() {
        let identity = Transform::default();

        assert_eq!(
            serde_json::from_value::<Transform>(json!([])).unwrap(),
            identity
        );
    }

    #[test]
    fn it_deserializes_single_transform() {
        let translation = Transform::default().translate(Vector3::from([1.0, 2.0, 3.0]));
        let rotation = Transform::default().rotate(Vector3::y_axis(), 50.0);
        let scale = Transform::default().scale(Vector3::from([1.0, 2.0, 3.0]));

        assert_eq!(
            serde_json::from_value::<Transform>(json!([
                { "translate": [1, 2, 3] }
            ]))
            .unwrap(),
            translation
        );
        assert_eq!(
            serde_json::from_value::<Transform>(json!([
                { "rotate": [[0, 1, 0], 50] }
            ]))
            .unwrap(),
            rotation
        );
        assert_eq!(
            serde_json::from_value::<Transform>(json!([
                { "scale": [1, 2, 3] }
            ]))
            .unwrap(),
            scale
        );
    }

    #[test]
    fn it_deserializes_complex_transforms() {
        let full = Transform::default()
            .rotate(Vector3::y_axis(), 50.0)
            .scale(Vector3::from([3.0, 2.0, 1.0]))
            .translate(Vector3::from([5.0, 2.0, 3.0]));
        let full_identity = Transform::default()
            .rotate(Vector3::y_axis(), 50.0)
            .scale(Vector3::from([1.0, 2.0, 4.0]))
            .translate(Vector3::from([1.0, 2.0, 3.0]))
            .translate(Vector3::from([-1.0, -2.0, -3.0]))
            .scale(Vector3::from([1.0, 0.5, 0.25]))
            .rotate(Vector3::y_axis(), -50.0);

        assert_eq!(
            serde_json::from_value::<Transform>(json!([
                { "rotate": [[0, 1, 0], 50] },
                { "scale": [3, 2, 1] },
                { "translate": [5, 2, 3] }
            ]))
            .unwrap(),
            full
        );
        assert_eq!(
            serde_json::from_value::<Transform>(json!([
                { "rotate": [[0, 1, 0], 50] },
                { "scale": [1, 2, 4] },
                { "translate": [1, 2, 3] },
                { "translate": [-1, -2, -3] },
                { "scale": [1, 0.5, 0.25] },
                { "rotate": [[0, 1, 0], -50] }
            ]))
            .unwrap(),
            full_identity
        );
    }
}
