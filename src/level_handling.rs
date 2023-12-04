use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui_kbgp::KbgpEguiUiCtxExt;
use bevy_pkv::PkvStore;
use bevy_yoleck::prelude::*;

use crate::menu::FocusLabel;
use crate::AppState;

pub struct LevelHandlingPlugin;

impl Plugin for LevelHandlingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelProgress>();
        app.add_systems(Update, read_last_finished_level);
        app.add_systems(
            OnEnter(AppState::LoadLevel),
            (unload_old_levels, launch_level_loading_command).chain(),
        );
        app.add_systems(OnEnter(AppState::LevelCompleted), handle_level_completion);
    }
}

#[derive(Resource, Default, Debug)]
pub struct LevelProgress {
    pub just_completed: Option<String>,
    pub current_level: Option<String>,
    pub num_levels_available: usize,
    pub level_index: Handle<YoleckLevelIndex>,
}

const LEVEL_PKV_KEY: &str = "completed_up_to_level";

fn read_last_finished_level(
    pkv: Res<PkvStore>,
    mut level_progress: ResMut<LevelProgress>,
    asset_server: Res<AssetServer>,
    level_index_assets: Res<Assets<YoleckLevelIndex>>,
) {
    if 0 < level_progress.num_levels_available {
        return;
    }
    level_progress.level_index = asset_server.load("levels/index.yoli");
    if let Ok(completed_up_to_level) = pkv.get::<String>(LEVEL_PKV_KEY) {
        let Some(level_index) = level_index_assets.get(&level_progress.level_index) else {
            return;
        };
        if let Some(index) = level_index.iter().enumerate().find_map(|(index, level)| {
            if level.filename == completed_up_to_level {
                Some(index)
            } else {
                None
            }
        }) {
            level_progress.num_levels_available = index + 2;
        } else {
            error!(
                "Unable to find level {:?}, starting anew",
                completed_up_to_level
            );
            level_progress.num_levels_available = 1;
        }
    } else {
        level_progress.num_levels_available = 1;
    }
}

fn unload_old_levels(query: Query<Entity, With<YoleckKeepLevel>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn launch_level_loading_command(
    level_progress: Res<LevelProgress>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    let current_level = level_progress
        .current_level
        .as_ref()
        .expect("`LoadLevel` state entered without setting `current_level`");
    commands.spawn(YoleckLoadLevel(
        asset_server.load(format!("levels/{}", current_level)),
    ));
    app_state.set(AppState::Game);
}

fn handle_level_completion(
    mut level_progress: ResMut<LevelProgress>,
    mut next_state: ResMut<NextState<AppState>>,
    level_index_assets: Res<Assets<YoleckLevelIndex>>,
    mut pkv: ResMut<PkvStore>,
    mut egui_contexts: EguiContexts,
) {
    let finished_level_name = level_progress
        .current_level
        .take()
        .expect("current_level should be set when entering the LevelCompleted state");
    if let Some(level_index) = level_index_assets.get(&level_progress.level_index) {
        let index_of_finished_level = level_index.iter().enumerate().find_map(|(index, level)| {
            if level.filename == finished_level_name {
                Some(index)
            } else {
                None
            }
        });
        if let Some(index_of_finished_level) = index_of_finished_level {
            let new_num_levels_available = index_of_finished_level + 2;
            if level_progress.num_levels_available < new_num_levels_available {
                level_progress.num_levels_available = new_num_levels_available;
                if let Err(err) = pkv.set_string(LEVEL_PKV_KEY, &finished_level_name) {
                    error!("Unable to save level progress: {}", err);
                }
            }
        }
    }
    level_progress.just_completed = Some(finished_level_name);
    egui_contexts
        .ctx_mut()
        .kbgp_set_focus_label(FocusLabel::NextLevel);
    next_state.set(AppState::LevelSelectMenu);
}
