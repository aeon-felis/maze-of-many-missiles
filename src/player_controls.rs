use bevy::prelude::*;
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
    });
}

fn apply_controls(
    mut query: Query<(
        &ActionState<PlayerAction>,
        &mut TnuaController,
        &mut PlayerFacing,
    )>,
) {
    for (input, mut controller, mut player_facing) in query.iter_mut() {
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
            // spring_strengh: todo!(),
            // spring_dampening: todo!(),
            // acceleration: todo!(),
            // air_acceleration: todo!(),
            // coyote_time: todo!(),
            // free_fall_extra_gravity: todo!(),
            // tilt_offset_angvel: todo!(),
            // tilt_offset_angacl: todo!(),
            // turning_angvel: todo!(),
            ..Default::default()
        });
        if let Some(jump) = Some(input.clamped_value(PlayerAction::Jump)).filter(|jump| 0.0 < *jump)
        {
            controller.action(TnuaBuiltinJump {
                height: 5.0 * jump,
                // allow_in_air: todo!(),
                // upslope_extra_gravity: todo!(),
                // takeoff_extra_gravity: todo!(),
                // takeoff_above_velocity: todo!(),
                // fall_extra_gravity: todo!(),
                // shorten_extra_gravity: todo!(),
                // peak_prevention_at_upward_velocity: todo!(),
                // peak_prevention_extra_gravity: todo!(),
                // reschedule_cooldown: todo!(),
                // input_buffer_time: todo!(),
                ..Default::default()
            });
        }
        // = Some(input.clamped_value(PlayerAction::Jump)).filter(|jump| 0.0 < *jump);
        // controls.jump = Some(input.clamped_value(PlayerAction::Jump)).filter(|jump| 0.0 < *jump);
    }
}
