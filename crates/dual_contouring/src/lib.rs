use glam::Vec3;

fn determinant(m: &[[f32; 3]; 3]) -> f32 {
    m[0][0] * m[1][1] * m[2][2] + m[0][1] * m[1][2] * m[2][0] + m[0][2] * m[1][0] * m[2][1]
        - m[0][2] * m[1][1] * m[2][0] - m[0][1] * m[1][0] * m[2][2] - m[0][0] * m[1][2] * m[2][1]
}

fn solve3x3(m: &[[f32; 3]; 3], b: &[f32; 3]) -> Option<[f32; 3]> {
    let det = determinant(m);
    if det.abs() <= std::f32::EPSILON {
        return None;
    }

    let mut x = [0.0_f32; 3];
    for i in 0..3 {
        let mut m2 = *m;
        for j in 0..3 {
            m2[j][i] = b[j];
        }
        x[i] = determinant(&m2) / det;
    }
    Some(x)
}
/// https://www.mattkeeter.com/projects/qef/
fn qef_solve(candidates: &[[f32; 4]]) -> Option<[f32; 3]> {
    let mut At_A = [[0.0_f32; 3];3];
    let mut At_b = [0.0_f32; 3];

    for i in 0..3 {
        for j in 0..3 {
            let mut sum = 0.0_f32;
            for k in 0..candidates.len() {
                sum += candidates[k][i] * candidates[k][j];
            }
            At_A[i][j] = sum;
        }
    }

    for i in 0..3 {
        let mut sum = 0.0_f32;
        for k in 0..candidates.len() {
            sum += candidates[k][i] * candidates[k][3];
        }
        At_b[i] = sum;
    }

    return solve3x3(&At_A, &At_b);
}

pub fn dual_contouring(
    density: Vec<f32>,
    width: usize,
    height: usize,
    depth: usize
) -> (Vec<[f32;3]>, Vec<[f32;3]>) {

    let corners = [
        (0, 0, 0),
        (0, 0, 1),
        (0, 1, 0),
        (0, 1, 1),
        (1, 0, 0),
        (1, 0, 1),
        (1, 1, 0),
        (1, 1, 1),
    ];
    let far_edges = [
        (3, 7),
        (5, 7),
        (6, 7)
    ];

    let index = |x: usize, y: usize, z: usize| {
        x + y * width + z * width * height
    };

    let mut vertices = vec![[0.0_f32; 3]; width * height * depth];

    for z in 0..depth-1 {
        for y in 0..height-1 {
            for x in 0..width-1 {
                let inside = corners.iter().map(|cornder| {
                    density[index(x + cornder.0, y + cornder.1, z + cornder.2)] <= 0.0
                })
                    .collect::<Vec<bool>>();

                let num_inside = inside.iter().filter(|&x| *x).count();
                if num_inside == 0 || num_inside == 8 {
                    continue;
                }

                let mut candidates = Vec::<[f32; 4]>::new();
                let mut mass_point = (0.0, 0.0, 0.0);

                for dx in 0..2 {
                    for dy in 0..2 {
                        let v0 = density[index(x + dx, y + dy, z)];
                        let v1 = density[index(x + dx, y + dy, z + 1)];

                        if (v0 > 0.0) != (v1 > 0.0) {
                            let t = v0 / (v0 - v1);
                            let p = (dx as f32, dy as f32, t);
                            candidates.push([p.0, p.1, p.2, 1.0]);
                        }
                    }
                }

                for dx in 0..2 {
                    for dz in 0..2 {
                        let v0 = density[index(x + dx, y, z + dz)];
                        let v1 = density[index(x + dx, y + 1, z + dz)];

                        if (v0 > 0.0) != (v1 > 0.0) {
                            let t = v0 / (v0 - v1);
                            let p = (dx as f32, t, dz as f32);
                            candidates.push([p.0, p.1, p.2, 1.0]);
                            mass_point.0 += p.0;
                            mass_point.1 += p.1;
                            mass_point.2 += p.2;
                        }
                    }
                }

                for dy in 0..2 {
                    for dz in 0..2 {
                        let v0 = density[index(x, y + dy, z + dz)];
                        let v1 = density[index(x + 1, y + dy, z + dz)];

                        if (v0 > 0.0) != (v1 > 0.0) {
                            let t = v0 / (v0 - v1);
                            let p = (t, dy as f32, dz as f32);
                            candidates.push([p.0, p.1, p.2, 1.0]);
                            mass_point.0 += p.0;
                            mass_point.1 += p.1;
                            mass_point.2 += p.2;
                        }
                    }
                }

                let num_candidates = candidates.len();
                if num_candidates == 0 {
                    continue;
                }

                mass_point.0 /= num_candidates as f32;
                mass_point.1 /= num_candidates as f32;
                mass_point.2 /= num_candidates as f32;

                // let bias_strength = 1.0;
                // let n = (bias_strength, 0.0, 0.0);

                let vertex = if let Some(vertex) = qef_solve(&candidates) {[
                    vertex[0].min(1.0).max(0.0),
                    vertex[1].min(1.0).max(0.0),
                    vertex[2].min(1.0).max(0.0),
                ]} else {
                    [0.5, 0.5, 0.5]
                };

                vertices[index(x, y, z)] = [
                    (x as f32 + vertex[0]) / width as f32,
                    (y as f32 + vertex[1]) / height as f32,
                    (z as f32 + vertex[2]) / depth as f32,
                ];
            }
        }
    }

    let mut mesh_positions = Vec::<[f32;3]>::new();
    let mut mesh_normals = Vec::<[f32;3]>::new();

    for z in 0..depth-2 {
        for y in 0..height-2 {
            for x in 0..width-2 {
                let inside = corners.iter().map(|corner| {
                    density[index(x + corner.0, y + corner.1, z + corner.2)] <= 0.0
                })
                    .collect::<Vec<bool>>();

                let v0 = Vec3::from(vertices[index(x, y, z)]);

                for face in 0..3 {
                    let e = far_edges[face];
                    if inside[e.0] == inside[e.1] {
                        continue;
                    }

                    let (v1, v2, v3) = match face {
                        0 => (
                            Vec3::from(vertices[index(x, y,   z+1)]),
                            Vec3::from(vertices[index(x, y+1, z)]),
                            Vec3::from(vertices[index(x, y+1, z+1)]),
                        ),
                        1 => (
                            Vec3::from(vertices[index(x, y,   z+1)]),
                            Vec3::from(vertices[index(x+1, y, z)]),
                            Vec3::from(vertices[index(x+1, y, z+1)]),
                        ),
                        2 => (
                            Vec3::from(vertices[index(x, y+1, z)]),
                            Vec3::from(vertices[index(x+1, y, z)]),
                            Vec3::from(vertices[index(x+1, y+1, z)]),
                        ),
                        _ => unreachable!(),
                    };

                    if inside[e.0] == (face == 1) {
                        mesh_positions.push(v0.into());
                        mesh_positions.push(v1.into());
                        mesh_positions.push(v3.into());

                        mesh_positions.push(v0.into());
                        mesh_positions.push(v3.into());
                        mesh_positions.push(v2.into());

                        let normal = (v1 - v0).cross(v3 - v0).normalize();

                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());

                        let normal = (v3 - v0).cross(v2 - v0).normalize();

                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());
                    }
                    else {
                        mesh_positions.push(v0.into());
                        mesh_positions.push(v3.into());
                        mesh_positions.push(v1.into());

                        mesh_positions.push(v0.into());
                        mesh_positions.push(v2.into());
                        mesh_positions.push(v3.into());

                        let normal = (v3 - v0).cross(v1 - v3).normalize();

                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());

                        let normal = (v2 - v0).cross(v3 - v0).normalize();

                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());
                        mesh_normals.push(normal.into());
                    }
                }
            }
        }
    }
    (mesh_positions, mesh_normals)
}