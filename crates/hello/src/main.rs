use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

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
                    title: "Alster".to_string(),
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

fn setup(mut commands: Commands) {
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
