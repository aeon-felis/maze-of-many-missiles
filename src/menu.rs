use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_egui_kbgp::prelude::*;
use bevy_yoleck::prelude::*;

use crate::level_handling::LevelProgress;
use crate::{ActionForKbgp, AppState, During};

#[derive()]
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameUi>();
        app.add_systems(Update, handle_user_kbgp_actions.in_set(During::Gameplay));
        app.add_systems(
            Update,
            (
                prepare_menu,
                menu_header,
                main_menu.run_if(in_state(AppState::MainMenu)),
                pause_menu.run_if(in_state(AppState::PauseMenu)),
                game_over_menu.run_if(in_state(AppState::GameOver)),
                level_select_menu.run_if(in_state(AppState::LevelSelectMenu)),
                #[cfg(not(target_arch = "wasm32"))]
                exit_button,
                draw_menu,
            )
                .chain()
                .in_set(During::Menu),
        );
    }
}

fn handle_user_kbgp_actions(
    mut egui_contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let egui_context = egui_contexts.ctx_mut();
    let Some(action) = egui_context.kbgp_user_action() else {
        return;
    };
    match action {
        ActionForKbgp::Menu => {
            next_state.set(AppState::PauseMenu);
        }
        ActionForKbgp::RestartLevel => {
            next_state.set(AppState::LoadLevel);
        }
    }
}

#[derive(PartialEq)]
pub enum FocusLabel {
    Start,
    Exit,
    NextLevel,
    BackToMainMenu,
    CurrentLevel,
}

#[derive(Resource, Default)]
struct FrameUi(Option<egui::Ui>);

fn prepare_menu(
    state: Res<State<AppState>>,
    mut egui_contexts: EguiContexts,
    mut frame_ui: ResMut<FrameUi>,
) {
    if state.get().is_menu() {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(egui_contexts.ctx_mut(), |ui| {
                let layout = egui::Layout::top_down(egui::Align::Center);
                ui.with_layout(layout, |ui| {
                    let frame = egui::Frame::none();
                    let mut prepared = frame.begin(ui);
                    let style = prepared.content_ui.style_mut();
                    style.text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId {
                            size: 32.0,
                            family: egui::FontFamily::Proportional,
                        },
                    );
                    style.spacing.item_spacing = egui::Vec2::new(10.0, 10.0);
                    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(32.0);
                    style.visuals.widgets.inactive.rounding = egui::Rounding::same(32.0);
                    frame_ui.0 = Some(prepared.content_ui);
                });
            });
    } else {
        frame_ui.0 = None;
    }
}

fn draw_menu(mut egui_contexts: EguiContexts, mut frame_ui: ResMut<FrameUi>) {
    let Some(frame_ui) = frame_ui.0.take() else {
        return;
    };
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_contexts.ctx_mut(), |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center);
            ui.with_layout(layout, |ui| {
                let frame = egui::Frame::none();
                let mut prepared = frame.begin(ui);
                prepared.content_ui = frame_ui;
                prepared.end(ui);
            });
        });
}

fn menu_header(mut frame_ui: ResMut<FrameUi>) {
    let Some(ui) = frame_ui.0.as_mut() else {
        return;
    };
    ui.add_space(50.0);

    let mut title_text = egui::text::LayoutJob::default();
    title_text.append(
        "Maze of Many Missiles",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId {
                size: 32.0,
                family: egui::FontFamily::Proportional,
            },
            color: egui::Color32::WHITE,
            ..Default::default()
        },
    );
    ui.label(title_text);
    ui.add_space(20.0);
}

fn format_level_name(filename: &str) -> String {
    filename
        .strip_suffix(".yol")
        .unwrap_or(filename)
        .replace('_', " ")
}

fn main_menu(mut frame_ui: ResMut<FrameUi>, mut next_state: ResMut<NextState<AppState>>) {
    let Some(ui) = frame_ui.0.as_mut() else {
        return;
    };
    if ui
        .button("Start")
        .kbgp_navigation()
        .kbgp_focus_label(FocusLabel::Start)
        .kbgp_initial_focus()
        .clicked()
    {
        next_state.set(AppState::LevelSelectMenu);
        ui.kbgp_clear_input();
        ui.kbgp_set_focus_label(FocusLabel::NextLevel);
    }
}

fn pause_menu(mut frame_ui: ResMut<FrameUi>, mut next_state: ResMut<NextState<AppState>>) {
    let Some(ui) = frame_ui.0.as_mut() else {
        return;
    };
    if ui
        .button("Resume")
        .kbgp_navigation()
        .kbgp_initial_focus()
        .kbgp_click_released()
        || ui.kbgp_user_action() == Some(ActionForKbgp::Menu)
    {
        next_state.set(AppState::Game);
    }
    if ui.button("Retry").kbgp_navigation().kbgp_click_released() {
        next_state.set(AppState::LoadLevel);
    }
    if ui
        .button("Level Select")
        .kbgp_navigation()
        .kbgp_initial_focus()
        .clicked()
    {
        next_state.set(AppState::LevelSelectMenu);
        ui.kbgp_clear_input();
        ui.kbgp_set_focus_label(FocusLabel::CurrentLevel);
    }
    if ui.button("Main Menu").kbgp_navigation().clicked() {
        next_state.set(AppState::MainMenu);
        ui.kbgp_clear_input();
        ui.kbgp_set_focus_label(FocusLabel::Start);
    }
}

fn game_over_menu(mut frame_ui: ResMut<FrameUi>, mut next_state: ResMut<NextState<AppState>>) {
    let Some(ui) = frame_ui.0.as_mut() else {
        return;
    };
    ui.label(
        egui::RichText::new("Game Over")
            .size(60.0)
            .strong()
            .color(egui::Color32::RED),
    );
    ui.add_space(20.0);
    if ui.kbgp_user_action() == Some(ActionForKbgp::Menu) {
        ui.kbgp_set_focus_label(FocusLabel::BackToMainMenu);
    }
    if ui
        .button("Retry")
        .kbgp_navigation()
        .kbgp_initial_focus()
        .clicked()
    {
        next_state.set(AppState::LoadLevel);
    }
    if ui
        .button("Level Select")
        .kbgp_navigation()
        .kbgp_initial_focus()
        .clicked()
    {
        next_state.set(AppState::LevelSelectMenu);
        ui.kbgp_clear_input();
        ui.kbgp_set_focus_label(FocusLabel::CurrentLevel);
    }
    if ui.button("Main Menu").kbgp_navigation().clicked() {
        next_state.set(AppState::MainMenu);
        ui.kbgp_clear_input();
        ui.kbgp_set_focus_label(FocusLabel::Start);
    }
}

fn level_select_menu(
    mut frame_ui: ResMut<FrameUi>,
    mut next_state: ResMut<NextState<AppState>>,
    level_index_assets: Res<Assets<YoleckLevelIndex>>,
    mut level_progress: ResMut<LevelProgress>,
) {
    let Some(ui) = frame_ui.0.as_mut() else {
        return;
    };

    if let Some(just_completed) = level_progress.just_completed.as_ref() {
        ui.label(
            egui::RichText::new(format!("Finished {}", format_level_name(just_completed)))
                .size(50.0)
                .color(egui::Color32::LIGHT_GREEN)
                .strong(),
        );
        ui.add_space(10.0);
    }

    let level_index = level_index_assets.get(&level_progress.level_index);

    if ui.kbgp_user_action() == Some(ActionForKbgp::Menu) {
        ui.kbgp_set_focus_label(FocusLabel::BackToMainMenu);
    }
    let mut response = ui
        .button("Back To Menu")
        .kbgp_navigation()
        .kbgp_focus_label(FocusLabel::BackToMainMenu);

    if level_index
        .map(|level_index| level_index.len() < level_progress.num_levels_available)
        .unwrap_or(true)
    {
        response = response.kbgp_focus_label(FocusLabel::NextLevel);
    }
    if response.clicked() {
        next_state.set(AppState::MainMenu);
    }

    let Some(level_index) = level_index else {
        return;
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (index, level) in level_index
            .iter()
            .enumerate()
            .take(level_progress.num_levels_available)
        {
            let mut button_text = egui::text::LayoutJob::default();
            button_text.append(
                &format_level_name(&level.filename),
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId {
                        size: 32.0,
                        family: egui::FontFamily::Proportional,
                    },
                    ..Default::default()
                },
            );
            if index + 1 < level_progress.num_levels_available {
                button_text.append(
                    "(complete)",
                    4.0,
                    egui::TextFormat {
                        font_id: egui::FontId {
                            size: 24.0,
                            family: egui::FontFamily::Proportional,
                        },
                        color: egui::Color32::GREEN,
                        ..Default::default()
                    },
                );
            }
            let mut response = ui.add(egui::Button::new(button_text)).kbgp_navigation();
            if index + 1 == level_progress.num_levels_available {
                response = response.kbgp_focus_label(FocusLabel::NextLevel);
            }
            if Some(&level.filename) == level_progress.current_level.as_ref() {
                response = response.kbgp_focus_label(FocusLabel::CurrentLevel);
            }
            if response.kbgp_click_released() {
                level_progress.current_level = Some(level.filename.clone());
                next_state.set(AppState::LoadLevel);
            }
        }
    });
}

#[allow(dead_code)]
fn exit_button(mut frame_ui: ResMut<FrameUi>, mut exit: EventWriter<bevy::app::AppExit>) {
    let Some(ui) = frame_ui.0.as_mut() else {
        return;
    };
    if ui
        .button("Exit")
        .kbgp_navigation()
        .kbgp_focus_label(FocusLabel::Exit)
        .clicked()
    {
        exit.send(Default::default());
    }
}
