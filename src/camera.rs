use bevy::prelude::*;
use bevy_yoleck::vpeol::prelude::*;

use crate::player::IsPlayer;
use crate::During;

pub struct MazeOfManyMissilesCameraPlugin;

impl Plugin for MazeOfManyMissilesCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(
            Update,
            (reset_camera_on_player_position, camera_track_player)
                .chain()
                .in_set(During::Gameplay),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 3.0, 30.0)
            .looking_to(Vec3::new(0.0, -3.0, -10.0), Vec3::Y),
        ..Default::default()
    });
    cmd.insert(VpeolCameraState::default());
    cmd.insert(Vpeol3dCameraControl::sidescroller());
    cmd.insert(PlayerTrackingCamera {
        camera_at: MovingPoint::new(10.0, 10.0, 5.0, 5.0),
        looking_at: MovingPoint::new(20.0, 20.0, 3.0, 5.0),
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 50_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 100.0, 0.0).looking_to(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });
}

struct MovingPoint {
    position: Vec2,
    velocity: Vec2,
    max_velocity: f32,
    acceleration: f32,
    stop_at_range: f32,
    stop_acceleration: f32,
}

impl MovingPoint {
    fn new(
        max_velocity: f32,
        acceleration: f32,
        stop_at_range: f32,
        stop_acceleration: f32,
    ) -> Self {
        Self {
            position: Default::default(),
            velocity: Vec2::ZERO,
            max_velocity,
            acceleration,
            stop_at_range,
            stop_acceleration,
        }
    }

    fn update_velocity(&mut self, desired_velocity: Vec2, max_impulse: f32) {
        let single_frame_impulse = desired_velocity - self.velocity;
        let actual_impulse = single_frame_impulse.clamp_length_max(max_impulse);
        self.velocity += actual_impulse;
    }

    fn update(&mut self, duration: f32, target: Vec2) {
        let vector_to_target = target - self.position;

        if self.stop_at_range.powi(2) < vector_to_target.length_squared() {
            self.update_velocity(
                vector_to_target.clamp_length_max(self.max_velocity),
                self.acceleration * duration,
            );
        } else {
            self.update_velocity(Vec2::ZERO, self.stop_acceleration * duration);
        }

        self.position += self.velocity * duration;
    }
}

#[derive(Component)]
struct PlayerTrackingCamera {
    camera_at: MovingPoint,
    looking_at: MovingPoint,
}

fn camera_track_player(
    time: Res<Time>,
    player_query: Query<&GlobalTransform, With<IsPlayer>>,
    mut cameras_query: Query<(&mut Transform, &mut PlayerTrackingCamera)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_position_2d = player_transform.translation().truncate();

    for (mut camera_transform, mut tracking) in cameras_query.iter_mut() {
        tracking
            .camera_at
            .update(time.delta_seconds(), player_position_2d);
        tracking
            .looking_at
            .update(time.delta_seconds(), player_position_2d);

        camera_transform.translation = tracking.looking_at.position.extend(30.0) + 3.0 * Vec3::Y;
        camera_transform.look_at(tracking.looking_at.position.extend(0.0), Vec3::Y);
    }
}

fn reset_camera_on_player_position(
    player_query: Query<&GlobalTransform, Added<IsPlayer>>,
    mut cameras_query: Query<&mut PlayerTrackingCamera>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_position_2d = player_transform.translation().truncate();

    for mut tracking in cameras_query.iter_mut() {
        let tracking = tracking.as_mut();
        for moving_point in [&mut tracking.camera_at, &mut tracking.looking_at] {
            moving_point.position = player_position_2d;
            moving_point.velocity = Vec2::ZERO;
        }
    }
}
