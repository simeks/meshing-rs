use std::time::{Duration, Instant};

/// Returns density and normals
fn generate_density(
    width: usize,
    height: usize,
    depth: usize,
) -> (Vec<f32>, Vec<glam::Vec3>)
{

    let mut densities = vec![0.0; width * height * depth];
    let mut normals = vec![glam::Vec3::ZERO; width * height * depth];

    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                densities[x + y * width + z * width * height] = glam::Vec3::new(
                    x as f32 - width as f32 / 2.0,
                    y as f32 - height as f32 / 2.0,
                    z as f32 - depth as f32 / 2.0,
                ).length() - 32.0;
                normals[x + y * width + z * width * height] = glam::Vec3::new(
                    x as f32, y as f32, z as f32
                ).normalize();
            }
        }
    }

    (densities, normals)
}


fn main() {

    let width = 128;
    let height = 128;
    let depth = 128;

    let (densities, normals) = generate_density(width, height, depth);

    let mut elapsed = vec![];

    for i in 0..100 {
        let begin = Instant::now();
        let _ = dual_contouring::dual_contouring(
            densities.clone(),
            normals.clone(),
            width,
            height,
            depth
        );
        if i > 0 {
            elapsed.push(Instant::now() - begin);
        }
    }

    let mean = elapsed.iter().sum::<Duration>() / elapsed.len() as u32;
    let std = elapsed.iter().map(|x| {
        let diff = x.as_millis() - mean.as_millis();
        diff * diff
    }).sum::<u128>() as f32 / elapsed.len() as f32;

    println!("Time: {:?} ms ({:?})", mean.as_millis(), std);
}