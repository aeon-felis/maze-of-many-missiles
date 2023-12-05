use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_yoleck::YoleckBelongsToLevel;
use ordered_float::OrderedFloat;

use crate::player::IsPlayer;
use crate::During;

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LaunchMissile>();
        app.add_systems(Update, launch_missiles);
        app.add_systems(
            Update,
            (propel_missiles, direct_missiles).in_set(During::Gameplay),
        );
    }
}

#[derive(Component)]
struct MissileConfig {
    speed: f32,
    acceleration: f32,
    angular_speed: f32,
    angular_acceleration: f32,
}

#[derive(Event, Debug)]
pub struct LaunchMissile {
    pub level: Entity,
    pub position: Vec2,
    pub direction: Vec2,
}

fn launch_missiles(
    mut reader: EventReader<LaunchMissile>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in reader.read() {
        let mut cmd = commands.spawn(SceneBundle {
            scene: asset_server.load("Missile.glb#Scene0"),
            transform: Transform {
                translation: event.position.extend(0.0),
                rotation: Quat::from_rotation_arc_2d(Vec2::X, event.direction),
                scale: Vec3::ONE,
            },
            ..Default::default()
        });
        cmd.insert(YoleckBelongsToLevel { level: event.level });

        cmd.insert(MissileConfig {
            speed: 30.0,
            acceleration: 400.0,
            angular_speed: 5.0,
            angular_acceleration: 100.0,
        });

        cmd.insert(RigidBody::Dynamic);
        cmd.insert(Collider::capsule_x(2.0, 0.25));
        cmd.insert(Velocity::linear(event.direction * 30.0));
    }
}

fn propel_missiles(
    time: Res<Time>,
    mut query: Query<(&MissileConfig, &mut Velocity, &GlobalTransform)>,
) {
    for (missile_config, mut velocity, transform) in query.iter_mut() {
        let direction = transform.right().truncate();
        let current_speed = velocity.linvel.dot(direction);
        let additional_speed_required = missile_config.speed - current_speed;
        if 0.0 < additional_speed_required {
            let boost =
                additional_speed_required.min(missile_config.acceleration * time.delta_seconds());
            velocity.linvel += boost * direction;
        }
    }
}

fn direct_missiles(
    time: Res<Time>,
    player_query: Query<&GlobalTransform, With<IsPlayer>>,
    mut missiles_query: Query<(&MissileConfig, &mut Velocity, &GlobalTransform)>,
) {
    if time.delta().is_zero() {
        return;
    }
    for (missile_config, mut velocity, transform) in missiles_query.iter_mut() {
        let missile_position = transform.translation().truncate();
        let Some(closest_player_position) = player_query
            .iter()
            .map(|t| t.translation().truncate())
            .min_by_key(|player_position| {
                OrderedFloat(player_position.distance_squared(missile_position))
            })
        else {
            continue;
        };
        let vector_to_target = missile_position - closest_player_position;
        let Some(direction_to_target) = vector_to_target.try_normalize() else {
            continue;
        };
        // TODO: compensate for gravity?
        let angle_diff = -transform
            .right()
            .truncate()
            .angle_between(direction_to_target);
        let desired_angvel = (angle_diff / time.delta_seconds())
            .clamp(-missile_config.angular_speed, missile_config.angular_speed);
        let angular_velocity_diff = desired_angvel - velocity.angvel;
        let maximum_impulse = missile_config.angular_acceleration * time.delta_seconds();
        let angular_impulse = angular_velocity_diff.clamp(-maximum_impulse, maximum_impulse);
        velocity.angvel += angular_impulse;
    }
}
