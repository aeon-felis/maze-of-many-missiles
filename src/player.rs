use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tnua::{prelude::*, TnuaAnimatingState, TnuaAnimatingStateDirective};
use bevy_tnua_rapier2d::TnuaRapier2dIOBundle;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol::VpeolWillContainClickableChildren;
use bevy_yoleck::vpeol_3d::Vpeol3dPosition;

use crate::animating::{AnimationsOwner, GetClipsFrom};
use crate::During;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Player")
                .with::<Vpeol3dPosition>()
                .insert_on_init(|| IsPlayer)
            //.insert_on_init(|| Vpeol3dRotatation(Quat::from_rotation_y(PI)))
            //.insert_on_init_during_editor(|| SnapToGrid)
        });
        app.add_systems(YoleckSchedule::Populate, populate_player);
        app.add_systems(
            Update,
            (set_player_facing, animate_player).in_set(During::Gameplay),
        );
    }
}

#[derive(Component)]
pub struct IsPlayer;

#[derive(Component, Debug)]
pub enum PlayerFacing {
    Left,
    Right,
}

impl PlayerFacing {
    pub fn direction(&self) -> Vec3 {
        match self {
            PlayerFacing::Left => Vec3::NEG_X,
            PlayerFacing::Right => Vec3::X,
        }
    }
}

#[derive(Component)]
struct RotationBasedOn(Entity);

fn populate_player(
    mut populate: YoleckPopulate<(), With<IsPlayer>>,
    asset_server: Res<AssetServer>,
) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_first_time() {
            cmd.insert(VpeolWillContainClickableChildren);
            let rotation_based_on = RotationBasedOn(cmd.id());
            let child = cmd
                .commands()
                .spawn((
                    SceneBundle {
                        scene: asset_server.load("Player.glb#Scene0"),
                        ..Default::default()
                    },
                    rotation_based_on,
                ))
                .id();
            cmd.add_child(child);
            // cmd.insert(ApplyRotationToChild(child));
            cmd.insert(AnimationsOwner::default());
            cmd.insert(GetClipsFrom(asset_server.load("Player.glb")));
        }
        cmd.insert(VisibilityBundle::default());
        cmd.insert(RigidBody::Dynamic);
        cmd.insert(Velocity::default());
        cmd.insert(Collider::capsule_y(0.25, 0.25));

        cmd.insert(TnuaControllerBundle::default());
        cmd.insert(TnuaRapier2dIOBundle::default());
        cmd.insert(LockedAxes::ROTATION_LOCKED);
        // cmd.insert(TnuaRapier2dSensorShape(Collider::cuboid(0.45, 0.0)));
        cmd.insert(ActiveEvents::COLLISION_EVENTS);
        // cmd.insert(SolverGroups {
        // memberships: crate::solver_groups::PLAYER,
        // filters: crate::solver_groups::PLANTED,
        // });

        cmd.insert(PlayerFacing::Right);

        // cmd.insert(Killable::default());
        cmd.insert(TnuaAnimatingState::<PlayerAnimationState>::default());
    });
}

fn set_player_facing(
    mut query: Query<(&mut Transform, &RotationBasedOn)>,
    players_query: Query<&PlayerFacing>,
) {
    for (mut transform, rotation_based_on) in query.iter_mut() {
        let Ok(facing) = players_query.get(rotation_based_on.0) else {
            continue;
        };
        transform.look_at(facing.direction(), Vec3::Y);
    }
}

#[derive(Debug)]
pub enum PlayerAnimationState {
    Standing,
    Running(f32),
    Jumping,
}

fn animate_player(
    mut query: Query<(
        &mut TnuaAnimatingState<PlayerAnimationState>,
        &TnuaController,
        &AnimationsOwner,
    )>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
) {
    for (mut animating_state, controller, animations_owner) in query.iter_mut() {
        let Some(animation_player) = animations_owner.players.get("Armature") else {
            continue;
        };
        let Ok(mut animation_player) = animation_players_query.get_mut(*animation_player) else {
            continue;
        };
        match animating_state.update_by_discriminant({
            match controller.action_name() {
                Some(TnuaBuiltinJump::NAME) => PlayerAnimationState::Jumping,
                Some(name) => panic!("Unknown action {name}"),
                None => {
                    let Some((_, walk_state)) = controller.concrete_basis::<TnuaBuiltinWalk>()
                    else {
                        continue;
                    };
                    let speed = walk_state.running_velocity.length();
                    if 0.1 < speed {
                        PlayerAnimationState::Running(0.35 * speed)
                    } else {
                        PlayerAnimationState::Standing
                    }
                }
            }
        }) {
            TnuaAnimatingStateDirective::Maintain { state } => {
                if let PlayerAnimationState::Running(speed) = state {
                    animation_player.set_speed(*speed);
                }
            }
            bevy_tnua::TnuaAnimatingStateDirective::Alter {
                old_state: _,
                state,
            } => match state {
                PlayerAnimationState::Standing => {
                    let Some(clip) = animations_owner.clips.get("Stand") else {
                        continue;
                    };
                    animation_player
                        .play_with_transition(clip.clone(), Duration::from_secs_f32(0.25))
                        .set_speed(1.0);
                }
                PlayerAnimationState::Running(speed) => {
                    let Some(clip) = animations_owner.clips.get("Walk") else {
                        continue;
                    };
                    animation_player
                        .play(clip.clone())
                        .repeat()
                        .set_speed(*speed);
                }
                PlayerAnimationState::Jumping => {
                    let Some(clip) = animations_owner.clips.get("Jump") else {
                        continue;
                    };
                    animation_player.play(clip.clone()).set_speed(3.0);
                }
            },
        }
    }
}
