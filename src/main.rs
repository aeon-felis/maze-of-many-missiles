// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_egui_kbgp::{KbgpNavBindings, KbgpNavCommand, KbgpPlugin, KbgpSettings};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_tnua::controller::TnuaControllerPlugin;
use bevy_tnua_rapier2d::TnuaRapier2dPlugin;
use bevy_yoleck::vpeol_3d::{Vpeol3dPluginForEditor, Vpeol3dPluginForGame};
use bevy_yoleck::{YoleckPluginForEditor, YoleckPluginForGame};
use clap::Parser;
use maze_of_many_missiles::{ActionForKbgp, MazeOfManyMissilesPlugin};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    editor: bool,
    #[clap(long)]
    level: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Maze of Many Missiles".to_owned(),
            resolution: (800.0, 600.0).into(),
            ..Default::default()
        }),
        ..Default::default()
    }));

    app.add_plugins(EguiPlugin);
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugins(TnuaControllerPlugin);
    app.add_plugins(TnuaRapier2dPlugin);

    if args.editor {
        app.add_plugins((
            YoleckPluginForEditor,
            Vpeol3dPluginForEditor::sidescroller(),
        ));
    } else {
        app.add_plugins((YoleckPluginForGame, Vpeol3dPluginForGame));
        app.add_plugins(KbgpPlugin);
        app.insert_resource(KbgpSettings {
            disable_default_navigation: true,
            disable_default_activation: false,
            prevent_loss_of_focus: true,
            focus_on_mouse_movement: true,
            allow_keyboard: true,
            allow_mouse_buttons: false,
            allow_mouse_wheel: false,
            allow_mouse_wheel_sideways: false,
            allow_gamepads: true,
            bindings: {
                KbgpNavBindings::default()
                    .with_wasd_navigation()
                    .with_key(KeyCode::Escape, KbgpNavCommand::user(ActionForKbgp::Menu))
                    .with_key(
                        KeyCode::Back,
                        KbgpNavCommand::user(ActionForKbgp::RestartLevel),
                    )
                    .with_key(KeyCode::Space, KbgpNavCommand::Click)
                    .with_key(KeyCode::J, KbgpNavCommand::Click)
                    .with_gamepad_button(
                        GamepadButtonType::Start,
                        KbgpNavCommand::user(ActionForKbgp::Menu),
                    )
                    .with_gamepad_button(
                        GamepadButtonType::Select,
                        KbgpNavCommand::user(ActionForKbgp::RestartLevel),
                    )
            },
        });
    }

    app.add_plugins(MazeOfManyMissilesPlugin {
        is_editor: args.editor,
        start_at_level: args.level,
    });

    app.run();
}
