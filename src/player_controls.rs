use bevy::prelude::*;
use bevy_tnua::control_helpers::TnuaSimpleAirActionsCounter;
use bevy_tnua::prelude::*;
use bevy_yoleck::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::player::{IsPlayer, PlayerFacing};
use crate::During;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    Run,
    Jump,
}

pub struct PlayerControlsPlugin;

impl Plugin for PlayerControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.add_systems(YoleckSchedule::Populate, add_controls_to_player);
        app.add_systems(Update, apply_controls.in_set(During::Gameplay));
    }
}

fn add_controls_to_player(mut populate: YoleckPopulate<(), With<IsPlayer>>) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_in_editor() {
            return;
        }
        cmd.insert(InputManagerBundle::<PlayerAction> {
            action_state: Default::default(),
            input_map: {
                let mut input_map = InputMap::default();

                input_map.insert(VirtualDPad::arrow_keys(), PlayerAction::Run);
                input_map.insert(VirtualDPad::wasd(), PlayerAction::Run);
                input_map.insert(VirtualDPad::dpad(), PlayerAction::Run);
                input_map.insert(DualAxis::left_stick(), PlayerAction::Run);

                input_map.insert(KeyCode::Z, PlayerAction::Jump);
                input_map.insert(KeyCode::J, PlayerAction::Jump);
                input_map.insert(GamepadButtonType::South, PlayerAction::Jump);

                input_map
            },
        });
        cmd.insert(PlayerAirCounters::default());
    });
}

#[derive(Component, Default)]
pub struct PlayerAirCounters {
    jumps: TnuaSimpleAirActionsCounter,
    // dashes: TnuaSimpleAirActionsCounter,
}

impl PlayerAirCounters {
    fn update(&mut self, controller: &TnuaController) {
        let Self { jumps } = self;
        jumps.update(controller);
    }
}

fn apply_controls(
    mut query: Query<(
        &ActionState<PlayerAction>,
        &mut TnuaController,
        &mut PlayerFacing,
        &mut PlayerAirCounters,
    )>,
) {
    for (input, mut controller, mut player_facing, mut air_counters) in query.iter_mut() {
        let controller = controller.as_mut();
        air_counters.update(controller);

        let desired_velocity = if let Some(axis_pair) = input.clamped_axis_pair(PlayerAction::Run) {
            if axis_pair.x() <= -0.1 {
                *player_facing = PlayerFacing::Left;
            } else if 0.1 <= axis_pair.x() {
                *player_facing = PlayerFacing::Right;
            }
            Vec3::X * 20.0 * axis_pair.x()
        } else {
            Vec3::ZERO
        };
        controller.basis(TnuaBuiltinWalk {
            desired_velocity,
            float_height: 1.5,
            cling_distance: 0.5,
            up: Vec3::Y,
            ..Default::default()
        });
        if let Some(jump) = Some(input.clamped_value(PlayerAction::Jump)).filter(|jump| 0.0 < *jump)
        {
            match air_counters.jumps.air_count_for(TnuaBuiltinJump::NAME) {
                1 => {
                    controller.named_action(
                        "air-jump",
                        TnuaBuiltinJump {
                            height: 4.0 * jump,
                            allow_in_air: true,
                            ..Default::default()
                        },
                    );
                }
                _ => {
                    controller.action(TnuaBuiltinJump {
                        height: 5.0 * jump,
                        allow_in_air: false,
                        ..Default::default()
                    });
                }
            }
        }
    }
}
