use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_yoleck::YoleckBelongsToLevel;

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LaunchMissile>();
        app.add_systems(Update, launch_missiles);
    }
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

        cmd.insert(RigidBody::Dynamic);
        cmd.insert(Collider::capsule_x(2.0, 0.25));
        cmd.insert(Velocity::linear(event.direction * 30.0));
    }
}
