use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::{CameraController, GeneratedMesh};

pub fn keyboard_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query_camera: Query<(&mut Transform, &CameraController)>,
    mut query_meshes: Query<(Entity, &mut Visibility, Option<&Wireframe>), With<GeneratedMesh>>,
) {
    for (mut transform, _) in query_camera.iter_mut() {

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
            velocity = 5.0;
        }

        direction = direction.normalize();

        if !direction.is_nan() {
            transform.translation += direction * velocity;
        }
    }

    if keyboard_input.just_pressed(KeyCode::Key1) {
        for (entity, _, wireframe) in query_meshes.iter_mut() {
            if wireframe.is_none() {
                commands.entity(entity).insert(Wireframe::default());
            } else {
                commands.entity(entity).remove::<Wireframe>();
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::Tab) {
        for (_, mut visibility, _) in query_meshes.iter_mut() {
            visibility.is_visible = !visibility.is_visible;
        }
    }
}

pub fn mouse_input(
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
