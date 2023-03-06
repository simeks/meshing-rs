use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use std::time::Instant;

#[derive(Component)]
struct CameraController {
    yaw: f32,
    pitch: f32,
}

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
        .add_startup_system(setup)
        .add_system(keyboard_input)
        .add_system(mouse_input)
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


    // for z in 0..depth {
    //     for y in 0..height {
    //         for x in 0..width {
    //             out[x + y * width + z * width * height] = Vec3::new(
    //                 x as f32 - width as f32 / 2.0,
    //                 y as f32 - height as f32 / 2.0,
    //                 z as f32 - depth as f32 / 2.0,
    //             ).length() - 16.0;
    //         }
    //     }
    // }

    (densities, normals)
}

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

    let width = 64;
    let height = 64;
    let depth = 64;

    let (densities, normals) = generate_density(width, height, depth);

    let begin = Instant::now();
    let (mesh_positions, mesh_normals) = dual_contouring::dual_contouring(
        densities,
        normals,
        width,
        height,
        depth
    );
    let end = Instant::now();
    println!("DC Time: {:?}", end - begin);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(mesh_positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(mesh_normals),
    );

    let mesh = meshes.add(mesh);
    commands.spawn((
        PbrBundle {
            mesh,
            material: materials.add(
                StandardMaterial {
                    base_color: Color::rgb(0.8, 0.8, 0.8),
                    reflectance: 0.0,
                    metallic: 0.0,
                    ..Default::default()
                }
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::new(100.0, 100.0, 100.0)),
            ..Default::default()
        },
    ));

}

fn keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &CameraController)>,
) {
    for (mut transform, _) in query.iter_mut() {

        let rotation = transform.rotation;
        
        let forward = rotation.mul_vec3(-Vec3::Z).normalize();
        let right = rotation.mul_vec3(Vec3::X).normalize();
        let up = Vec3::Y;

        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::W) {
            direction += forward;
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction -= forward;
        }
        if keyboard_input.pressed(KeyCode::A) {
            direction -= right;
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction += right;
        }
        if keyboard_input.pressed(KeyCode::Q) {
            direction -= up;
        }
        if keyboard_input.pressed(KeyCode::E) {
            direction += up;
        }
        let mut velocity = 0.1;
        if keyboard_input.pressed(KeyCode::LShift) {
            velocity = 10.0;
        }

        direction = direction.normalize();

        if !direction.is_nan() {
            transform.translation += direction * velocity;
        }
    }
}

fn mouse_input(
    mouse_input: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController, )>,
) {
    for ev in motion_evr.iter() {
        if mouse_input.pressed(MouseButton::Left) {
            for (mut transform, mut controller) in query.iter_mut() {
                controller.yaw -= ev.delta.x * 0.1;
                controller.pitch -= ev.delta.y * 0.1;

                controller.pitch = controller.pitch.clamp(-90.0, 90.0);

                let yaw_radians = controller.yaw.to_radians();
                let pitch_radians = controller.pitch.to_radians();

                transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
                    * Quat::from_axis_angle(Vec3::X, pitch_radians);
            }
        }
    }
}
