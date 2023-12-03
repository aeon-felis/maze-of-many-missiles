use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::utils::HashMap;

pub struct AnimatingPlugin;

impl Plugin for AnimatingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                detect_animation_players,
                detect_animation_clips,
                set_initial_animation,
            ),
        );
    }
}

#[derive(Component, Default, Debug)]
pub struct AnimationsOwner {
    pub players: HashMap<String, Entity>,
    pub clips: HashMap<String, Handle<AnimationClip>>,
}

#[derive(Component)]
pub struct GetClipsFrom(pub Handle<Gltf>);

#[derive(Component)]
pub struct InitialAnimation {
    player_name: String,
    clip_name: String,
}

#[allow(dead_code)]
impl InitialAnimation {
    pub fn new(player_name: impl ToString, clip_name: impl ToString) -> Self {
        Self {
            player_name: player_name.to_string(),
            clip_name: clip_name.to_string(),
        }
    }
}

fn detect_animation_players(
    query: Query<(Entity, &Name), Added<AnimationPlayer>>,
    parents_query: Query<&Parent>,
    mut animation_owners_query: Query<&mut AnimationsOwner>,
) {
    for (entity, name) in query.iter() {
        let mut parent_entity = entity;
        while let Ok(parent) = parents_query.get(parent_entity) {
            parent_entity = **parent;
            if let Ok(mut animation_owners) = animation_owners_query.get_mut(parent_entity) {
                animation_owners
                    .players
                    .insert(name.as_str().to_owned(), entity);
                break;
            }
        }
    }
}

fn detect_animation_clips(
    mut query: Query<(Entity, &mut AnimationsOwner, &GetClipsFrom)>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for (entity, mut animation_owners, GetClipsFrom(get_clips_from)) in query.iter_mut() {
        let Some(gltf) = gltf_assets.get(get_clips_from) else {
            continue;
        };
        commands.entity(entity).remove::<GetClipsFrom>();
        for (name, clip) in gltf.named_animations.iter() {
            animation_owners.clips.insert(name.clone(), clip.clone());
        }
    }
}

fn set_initial_animation(
    query: Query<(Entity, &InitialAnimation, &AnimationsOwner)>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
    mut commands: Commands,
) {
    for (entity, initial_animation, animations_owner) in query.iter() {
        let Some(animation_clip) = animations_owner.clips.get(&initial_animation.clip_name) else {
            continue;
        };
        let Some(animation_player) = animations_owner.players.get(&initial_animation.player_name)
        else {
            continue;
        };
        let Ok(mut animation_player) = animation_players_query.get_mut(*animation_player) else {
            continue;
        };
        animation_player.play(animation_clip.clone()).repeat();
        commands.entity(entity).remove::<InitialAnimation>();
    }
}
