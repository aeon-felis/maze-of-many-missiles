use bevy::prelude::*;
use bevy_yoleck::prelude::*;
use bevy_yoleck::vpeol::VpeolWillContainClickableChildren;
use bevy_yoleck::vpeol_3d::{Vpeol3dPosition, Vpeol3dRotatation};

pub struct CannonPlugin;

impl Plugin for CannonPlugin {
    fn build(&self, app: &mut App) {
        app.add_yoleck_entity_type({
            YoleckEntityType::new("Cannon")
                .with::<Vpeol3dPosition>()
                .with::<Vpeol3dRotatation>()
                .insert_on_init(|| IsCannon)
        });

        app.add_systems(YoleckSchedule::Populate, populate_cannon);
        app.add_yoleck_edit_system(edit_cannon_direction)
    }
}

#[derive(Component)]
pub struct IsCannon;

fn populate_cannon(
    mut populate: YoleckPopulate<(), With<IsCannon>>,
    asset_server: Res<AssetServer>,
) {
    populate.populate(|ctx, mut cmd, ()| {
        if ctx.is_first_time() {
            cmd.insert(VpeolWillContainClickableChildren);
            cmd.insert(SceneBundle {
                scene: asset_server.load("Cannon.glb#Scene0"),
                ..Default::default()
            });
        }
    });
}

fn edit_cannon_direction(
    mut edit: YoleckEdit<(Entity, &mut Vpeol3dRotatation, &Vpeol3dPosition), With<IsCannon>>,
    mut knobs: YoleckKnobs,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_and_material: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
) {
    for (cannon_entity, mut cannon_rotation, cannon_position) in edit.iter_matching_mut() {
        let unnormalized_direction = (cannon_rotation.0 * Vec3::NEG_Z).truncate();
        let direction = unnormalized_direction.try_normalize().unwrap_or(Vec2::Y);
        if 0.1 < direction.distance_squared(unnormalized_direction) {
            cannon_rotation.0 = Quat::from_rotation_arc(Vec3::NEG_Z, direction.extend(0.0));
        }

        let mut knob = knobs.knob(("cannon-direction", cannon_entity));

        if knob.is_new {
            let (knob_mesh, knob_material) = mesh_and_material
                .get_or_insert_with(|| {
                    (
                        meshes.add(Mesh::from(shape::UVSphere {
                            radius: 0.4,
                            sectors: 10,
                            stacks: 10,
                        })),
                        materials.add(Color::PINK.into()),
                    )
                })
                .clone();
            knob.cmd.insert(PbrBundle {
                mesh: knob_mesh,
                material: knob_material,
                ..Default::default()
            });
        }
        knob.cmd.insert(Transform::from_translation(
            cannon_position.0 + 2.0 * direction.extend(0.0),
        ));
        if let Some(new_marker_pos) = knob.get_passed_data::<Vec3>() {
            let desired_direction = (*new_marker_pos - cannon_position.0)
                .truncate()
                .normalize_or_zero();
            let new_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, desired_direction.extend(0.0));
            cannon_rotation.0 = new_rotation;
        }
    }
}
