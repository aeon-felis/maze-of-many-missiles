use bevy::prelude::*;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol::VpeolWillContainClickableChildren;
use bevy_yoleck::vpeol_3d::{Vpeol3dPosition, Vpeol3dRotatation};

use crate::animating::{AnimationsOwner, GetClipsFrom, InitialAnimation};
use crate::utils::CachedPbrMaker;

pub struct ArrowPlugin;

impl Plugin for ArrowPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Arrow")
                .with::<Vpeol3dPosition>()
                .with::<Vpeol3dRotatation>()
                .insert_on_init(|| IsArrow)
        });

        app.add_systems(YoleckSchedule::Populate, populate_arrow);
        app.add_yoleck_edit_system(edit_arrow_direction_and_z_position);
    }
}

#[derive(Component)]
pub struct IsArrow;

#[derive(Component)]
pub struct FireEvery(Timer);

fn populate_arrow(mut populate: YoleckPopulate<(), With<IsArrow>>, asset_server: Res<AssetServer>) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_first_time() {
            cmd.insert(VpeolWillContainClickableChildren);
            cmd.insert(SceneBundle {
                scene: asset_server.load("Arrow.glb#Scene0"),
                ..Default::default()
            });
            cmd.insert(AnimationsOwner::default());
            cmd.insert(GetClipsFrom(asset_server.load("Arrow.glb")));
            cmd.insert(InitialAnimation::new("Armature", "Dance", 2.0));
        }
    });
}

fn edit_arrow_direction_and_z_position(
    mut edit: YoleckEdit<(Entity, &mut Vpeol3dRotatation, &mut Vpeol3dPosition), With<IsArrow>>,
    mut knobs: YoleckKnobs,
    mut pbr: CachedPbrMaker,
) {
    for (arrow_entity, mut arrow_rotation, mut arrow_position) in edit.iter_matching_mut() {
        arrow_position.0.z = -5.0;
        let unnormalized_direction = (arrow_rotation.0 * Vec3::X).truncate();
        let direction = unnormalized_direction.try_normalize().unwrap_or(Vec2::X);

        let mut knob = knobs.knob(("arrow-direction", arrow_entity));

        if knob.is_new {
            knob.cmd.insert(pbr.make_pbr_with(
                || {
                    Mesh::from(shape::UVSphere {
                        radius: 0.4,
                        sectors: 10,
                        stacks: 10,
                    })
                },
                || Color::PINK.into(),
            ));
        }
        knob.cmd.insert(Transform::from_translation(
            arrow_position.0 + 3.0 * direction.extend(0.0),
        ));
        if let Some(new_marker_pos) = knob.get_passed_data::<Vec3>() {
            let desired_direction = (*new_marker_pos - arrow_position.0)
                .truncate()
                .normalize_or_zero();
            let new_rotation = Quat::from_rotation_arc_2d(Vec2::X, desired_direction);
            arrow_rotation.0 = new_rotation;
        }
    }
}
