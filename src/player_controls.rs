use std::time::Duration;

use bevy::prelude::*;
use bevy_tnua::builtins::TnuaBuiltinDash;
use bevy_tnua::control_helpers::TnuaAirActionsTracker;
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

                input_map.insert(KeyCode::Space, PlayerAction::Jump);
                input_map.insert(KeyCode::J, PlayerAction::Jump);
                input_map.insert(GamepadButtonType::South, PlayerAction::Jump);

                input_map
            },
        });
        cmd.insert(PlayerAirCounters::default());
        cmd.insert(DoubleClickInputs::default());
    });
}

#[derive(Default)]
enum CurrentAirAction {
    #[default]
    None,
    Jump,
    Dash,
}

#[derive(Component, Default)]
struct PlayerAirCounters {
    tracker: TnuaAirActionsTracker,
    current: CurrentAirAction,
    jumps: usize,
    dashes: usize,
}

impl PlayerAirCounters {
    fn update(&mut self, controller: &TnuaController) {
        match self.tracker.update(controller) {
            bevy_tnua::control_helpers::TnuaAirActionsUpdate::NoChange => {}
            bevy_tnua::control_helpers::TnuaAirActionsUpdate::FreeFallStarted => {
                self.current = CurrentAirAction::None;
            }
            bevy_tnua::control_helpers::TnuaAirActionsUpdate::AirActionStarted(action) => {
                match action {
                    TnuaBuiltinJump::NAME | "air-jump" => {
                        self.current = CurrentAirAction::Jump;
                        self.jumps += 1;
                    }
                    TnuaBuiltinDash::NAME => {
                        self.current = CurrentAirAction::Dash;
                        self.dashes += 1;
                    }
                    _ => {}
                }
            }
            bevy_tnua::control_helpers::TnuaAirActionsUpdate::ActionFinishedInAir => {
                self.current = CurrentAirAction::None;
            }
            bevy_tnua::control_helpers::TnuaAirActionsUpdate::JustLanded => {
                self.current = CurrentAirAction::None;
                self.jumps = 0;
                self.dashes = 0;
            }
        }
    }

    fn jump_count(&self) -> usize {
        if matches!(self.current, CurrentAirAction::Jump) {
            self.jumps - 1
        } else {
            self.jumps
        }
    }

    fn dash_count(&self) -> usize {
        if matches!(self.current, CurrentAirAction::Dash) {
            self.dashes - 1
        } else {
            self.dashes
        }
    }
}

#[derive(Default)]
enum DoubleClickDetector {
    #[default]
    Idle,
    Pressed,
    Maybe,
    Active,
    MaybeActive,
    Pending(Duration),
}

impl DoubleClickDetector {
    fn update(&mut self, time_delta: Duration) {
        match self {
            Self::Idle => {}
            Self::Pressed => {
                *self = Self::Maybe;
            }
            Self::Maybe => {
                *self = Self::Pending(time_delta);
            }
            Self::Active => {
                *self = Self::MaybeActive;
            }
            Self::MaybeActive => {
                *self = Self::Idle;
            }
            Self::Pending(duration) => {
                *duration += time_delta;
            }
        }
    }

    fn update_pressed(&mut self) {
        match self {
            Self::Idle | Self::Maybe => {
                *self = Self::Pressed;
            }
            Self::Pressed | Self::Active => {}
            Self::MaybeActive => {
                *self = Self::Active;
            }
            Self::Pending(duration) => {
                *self = if duration.as_secs_f64() < 0.1 {
                    Self::Active
                } else {
                    Self::Pressed
                };
            }
        }
    }

    fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Component, Default)]
struct DoubleClickInputs {
    left: DoubleClickDetector,
    right: DoubleClickDetector,
}

impl DoubleClickInputs {
    fn update(&mut self, time_delta: Duration) {
        let Self { left, right } = self;
        left.update(time_delta);
        right.update(time_delta);
    }
}

fn apply_controls(
    time: Res<Time>,
    mut query: Query<(
        &ActionState<PlayerAction>,
        &mut TnuaController,
        &mut PlayerFacing,
        &mut PlayerAirCounters,
        &mut DoubleClickInputs,
    )>,
) {
    for (input, mut controller, mut player_facing, mut air_counters, mut double_click_inputs) in
        query.iter_mut()
    {
        let controller = controller.as_mut();
        air_counters.update(controller);
        double_click_inputs.update(time.delta());

        let desired_velocity = if let Some(axis_pair) = input.clamped_axis_pair(PlayerAction::Run) {
            if axis_pair.x() <= -0.1 {
                *player_facing = PlayerFacing::Left;
                double_click_inputs.left.update_pressed();
            } else if 0.1 <= axis_pair.x() {
                *player_facing = PlayerFacing::Right;
                double_click_inputs.right.update_pressed();
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
            match air_counters.jump_count() {
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

        for (double_click_input, direction_x) in [
            (&double_click_inputs.left, -1.0),
            (&double_click_inputs.right, 1.0),
        ] {
            if double_click_input.is_active() {
                controller.action(TnuaBuiltinDash {
                    displacement: 10.0 * direction_x * Vec3::X,
                    // desired_forward: todo!(),
                    allow_in_air: air_counters.dash_count() < 1,
                    speed: 120.0,
                    brake_to_speed: 40.0,
                    acceleration: 800.0,
                    // brake_acceleration: todo!(),
                    // input_buffer_time: todo!(),
                    ..Default::default()
                });
            }
        }
    }
}
