use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol_3d::{Vpeol3dPosition, Vpeol3dRotatation, Vpeol3dScale};

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Block")
                .with::<Vpeol3dPosition>()
                .with::<Vpeol3dScale>()
                .with::<Vpeol3dRotatation>()
                .insert_on_init(|| IsBlock)
        });

        app.add_yoleck_edit_system(resize_block);
        app.add_yoleck_edit_system(rotate_block);

        app.add_systems(YoleckSchedule::Populate, populate_block);
    }
}

#[derive(Component)]
pub struct IsBlock;

fn populate_block(
    mut populate: YoleckPopulate<(), With<IsBlock>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_and_material: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_first_time() {
            let (mesh, material) = mesh_and_material
                .get_or_insert_with(|| {
                    (
                        meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0))),
                        materials.add(Color::GRAY.into()),
                    )
                })
                .clone();
            cmd.insert(PbrBundle {
                mesh,
                material,
                ..Default::default()
            });
            cmd.insert(RigidBody::Fixed);
            cmd.insert(Collider::cuboid(0.5, 0.5));
        }
    });
}

fn resize_block(
    mut edit: YoleckEdit<
        (&Vpeol3dRotatation, &mut Vpeol3dScale, &mut Vpeol3dPosition),
        With<IsBlock>,
    >,
    mut knobs: YoleckKnobs,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_and_material: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
) {
    let Ok((rotation, mut scale, mut position)) = edit.get_single_mut() else {
        return;
    };

    let (knob_mesh, knob_material) = mesh_and_material
        .get_or_insert_with(|| {
            (
                meshes.add(Mesh::from(shape::Box::new(0.4, 0.4, 1.1))),
                materials.add(Color::ORANGE.into()),
            )
        })
        .clone();

    for (i, diagonal) in [
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = 0.5 * diagonal * scale.0.truncate();
        let mut knob = knobs.knob(("resize-marker", i));
        if knob.is_new {
            knob.cmd.insert(PbrBundle {
                mesh: knob_mesh.clone(),
                material: knob_material.clone(),
                ..Default::default()
            });
        }
        knob.cmd.insert(Transform::from_translation(
            position.0 + rotation.0 * offset.extend(0.0),
        ));

        if let Some(new_marker_pos) = knob.get_passed_data::<Vec3>() {
            let inverse_rotation = rotation.0.inverse();
            let other_corner = position.0 - (inverse_rotation * offset.extend(0.0));
            let size_f = (*new_marker_pos - other_corner).truncate();
            let size_f = size_f * diagonal;
            let size_f = Vec2::from_array(size_f.to_array().map(|coord| coord.max(0.0)));
            scale.0 = size_f.extend(1.0);
            position.0 = other_corner + 0.5 * (inverse_rotation * (diagonal * size_f).extend(0.0));
        }
    }
}

fn rotate_block(
    mut edit: YoleckEdit<
        (&mut Vpeol3dRotatation, &Vpeol3dScale, &mut Vpeol3dPosition),
        With<IsBlock>,
    >,
    mut knobs: YoleckKnobs,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_and_material: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
) {
    let Ok((mut rotation, scale, position)) = edit.get_single_mut() else {
        return;
    };

    let (knob_mesh, knob_material) = mesh_and_material
        .get_or_insert_with(|| {
            (
                meshes.add(Mesh::from(shape::UVSphere {
                    radius: 0.4,
                    sectors: 10,
                    stacks: 10,
                })),
                materials.add(Color::GREEN.into()),
            )
        })
        .clone();

    for (i, knob_direction) in [
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(0.0, -1.0),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = 0.5 * knob_direction * scale.0.truncate();
        let rotated_offset = rotation.0 * offset.extend(0.0);
        let mut knob = knobs.knob(("rotate-marker", i));
        if knob.is_new {
            knob.cmd.insert(PbrBundle {
                mesh: knob_mesh.clone(),
                material: knob_material.clone(),
                ..Default::default()
            });
        }
        knob.cmd
            .insert(Transform::from_translation(position.0 + rotated_offset));

        if let Some(new_marker_pos) = knob.get_passed_data::<Vec3>() {
            let desired_direction = (*new_marker_pos - position.0)
                .truncate()
                .normalize_or_zero();
            let new_rotation = Quat::from_rotation_arc_2d(knob_direction, desired_direction);
            rotation.0 = new_rotation;
        }
    }
}
