use nalgebra::{Matrix3, SymmetricEigen, Vector3};

pub fn fit_plane_pca(
    points: &[[f32; 3]],
) -> Option<(Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector3<f32>)> {
    if points.len() < 3 {
        return None;
    }

    let mut mean = Vector3::zeros();
    for p in points {
        mean.x += p[0];
        mean.y += p[1];
        mean.z += p[2];
    }
    mean /= points.len() as f32;

    let mut cov = Matrix3::zeros();
    for p in points {
        let v = Vector3::new(p[0], p[1], p[2]) - mean;
        cov[(0, 0)] += v.x * v.x;
        cov[(0, 1)] += v.x * v.y;
        cov[(0, 2)] += v.x * v.z;
        cov[(1, 0)] += v.y * v.x;
        cov[(1, 1)] += v.y * v.y;
        cov[(1, 2)] += v.y * v.z;
        cov[(2, 0)] += v.z * v.x;
        cov[(2, 1)] += v.z * v.y;
        cov[(2, 2)] += v.z * v.z;
    }

    let eig: SymmetricEigen<f32, nalgebra::Const<3>> = SymmetricEigen::new(cov);
    let mut min_idx = 0usize;
    let mut min_val = eig.eigenvalues[0];
    for (i, val) in eig.eigenvalues.iter().enumerate().skip(1) {
        if *val < min_val {
            min_val = *val;
            min_idx = i;
        }
    }
    let normal = eig.eigenvectors.column(min_idx).normalize();

    let reference = if normal.z.abs() < 0.9 {
        Vector3::new(0.0, 0.0, 1.0)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };
    let u_axis = normal.cross(&reference).normalize();
    let v_axis = normal.cross(&u_axis).normalize();

    Some((mean, u_axis, v_axis, normal))
}
