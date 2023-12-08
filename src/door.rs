use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol::VpeolWillContainClickableChildren;
use bevy_yoleck::vpeol_3d::Vpeol3dPosition;

use crate::player::IsPlayer;
use crate::utils::collision_started_events_both_ways;
use crate::AppState;

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Door")
                .with::<Vpeol3dPosition>()
                .insert_on_init(|| IsDoor)
        });

        app.add_systems(YoleckSchedule::Populate, populate_door);
        app.add_systems(Update, player_enter_door);
    }
}

#[derive(Component)]
pub struct IsDoor;

fn populate_door(mut populate: YoleckPopulate<(), With<IsDoor>>, asset_server: Res<AssetServer>) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_first_time() {
            cmd.insert(VpeolWillContainClickableChildren);
            cmd.insert(SceneBundle {
                scene: asset_server.load("Door.glb#Scene0"),
                ..Default::default()
            });
        }
        cmd.insert(Collider::cuboid(2.0, 2.0));
        cmd.insert(Sensor);
    });
}

fn player_enter_door(
    mut reader: EventReader<CollisionEvent>,
    player_query: Query<(), With<IsPlayer>>,
    door_query: Query<(), With<IsDoor>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (e1, e2) in collision_started_events_both_ways(&mut reader) {
        if player_query.contains(e1) && door_query.contains(e2) {
            next_state.set(AppState::LevelCompleted);
        }
    }
}
