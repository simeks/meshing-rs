mod input;

use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use std::time::Instant;

// Size of isosurface field
const FIELD_WIDTH: usize = 128;
const FIELD_HEIGHT: usize = 128;
const FIELD_DEPTH: usize = 128;

#[derive(Component)]
pub struct CameraController {
    yaw: f32,
    pitch: f32,
}

#[derive(Component)]
pub struct GeneratedMesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "Hello".to_string(),
                    width: 1280.0,
                    height: 720.0,
                    ..Default::default()
                },
                ..default()
            }))
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_system(input::keyboard_input)
        .add_system(input::mouse_input)
        .run();
}

/// Returns density and normals
fn generate_density(
    width: usize,
    height: usize,
    depth: usize,
) -> (Vec<f32>, Vec<glam::Vec3>)
{
    // Can't use NoiseBuilder: https://github.com/verpeteren/rust-simd-noise/issues/38

    let mut densities = vec![0.0; width * height * depth];
    let mut normals = vec![glam::Vec3::ZERO; width * height * depth];

    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let index = x + y * width + z * width * height;
                let d = unsafe { simdnoise::scalar::fbm_3d(
                    x as f32 / 32.0,
                    y as f32 / 32.0,
                    z as f32 / 32.0,
                    0.1,
                    2.0,
                    5,
                    1234
                ) };

                densities[index] = d;
                normals[index] = glam::Vec3::new(
                    unsafe { simdnoise::scalar::fbm_3d(
                        x as f32 / 32.0 + 0.1,
                        y as f32 / 32.0,
                        z as f32 / 32.0,
                        0.1,
                        2.0,
                        5,
                        1234
                    ) } - d,
                    unsafe { simdnoise::scalar::fbm_3d(
                        x as f32 / 32.0,
                        y as f32 / 32.0 + 0.1,
                        z as f32 / 32.0,
                        0.1,
                        2.0,
                        5,
                        1234
                    ) } - d,
                    unsafe { simdnoise::scalar::fbm_3d(
                        x as f32 / 32.0,
                        y as f32 / 32.0,
                        z as f32 / 32.0 + 0.1,
                        0.1,
                        2.0,
                        5,
                        1234
                    ) } - d,
                ).normalize();
            }
        }
    }
    (densities, normals)
}

fn create_mesh(positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh
}

/// Setup scene, including generating marching cubes and dual contouring meshes
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut transform = Transform::from_xyz(0.0, 100.0, 0.0);

    let yaw: f32 = -135.0;
    let pitch: f32 = 0.0;

    transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw.to_radians())
    * Quat::from_axis_angle(Vec3::X, pitch.to_radians());

    commands.spawn(Camera3dBundle {
        projection: Projection::Perspective(
                PerspectiveProjection {
                fov: std::f32::consts::PI / 4.0,
                near: 0.1,
                far: 20000.0,
                aspect_ratio: 1.0,
            }
        ),
        transform,
        ..default()
    })
        .insert(CameraController {
            yaw,
            pitch,
        });

    commands.spawn(DirectionalLightBundle {
        transform: Transform {
            rotation: Quat::from_rotation_y(-135.0_f32.to_radians()) * Quat::from_rotation_x(-45.0_f32.to_radians()),
            ..default()
        },
        ..Default::default()
    });

    // Generate isosurface

    let (densities, normals) = generate_density(FIELD_WIDTH, FIELD_HEIGHT, FIELD_DEPTH);

    // Dual contouring

    let begin = Instant::now();
    let (mesh_positions, mesh_normals) = meshing::dual_contouring(
        &densities,
        &normals,
        FIELD_WIDTH,
        FIELD_HEIGHT,
        FIELD_DEPTH,
    );
    let end = Instant::now();
    println!("DC Time: {:?}", end - begin);

    let mesh = meshes.add(create_mesh(mesh_positions, mesh_normals));
    commands.spawn((
        PbrBundle {
            mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.0, 0.0),
                metallic: 0.0,
                reflectance: 0.0,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(50.0, 50.0, 50.0))
                .with_scale(Vec3::new(100.0, 100.0, 100.0)),
            ..Default::default()
        },
    ))
        .insert(Wireframe)
        .insert(GeneratedMesh);

    // Marching cubes

    let begin = Instant::now();
    let (mesh_positions, mesh_normals) = meshing::marching_cubes(
        &densities,
        FIELD_WIDTH,
        FIELD_HEIGHT,
        FIELD_DEPTH,
    );
    let end = Instant::now();
    println!("MC Time: {:?}", end - begin);

    let mesh = meshes.add(create_mesh(mesh_positions, mesh_normals));
    commands.spawn((
        PbrBundle {
            mesh,
            visibility: Visibility { is_visible: false },
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.8, 0.0),
                metallic: 0.0,
                reflectance: 0.0,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(50.0, 50.0, 50.0))
                .with_scale(Vec3::new(100.0, 100.0, 100.0)),
            ..Default::default()
        },
    ))
        .insert(Wireframe)
        .insert(GeneratedMesh);
}
