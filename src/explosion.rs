use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_yoleck::YoleckBelongsToLevel;

use crate::utils::CachedPbrMaker;
use crate::During;

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartExplosion>();
        app.add_systems(Update, start_explosions);
        app.add_systems(
            Update,
            (progress_explosion_lifetime, apply_explosion_force).in_set(During::Gameplay),
        );
    }
}

#[derive(Event, Debug)]
pub struct StartExplosion {
    pub level: Entity,
    pub position: Vec2,
}

#[derive(Component)]
struct ExplosionStatus {
    timer: Timer,
}

#[derive(Component)]
pub struct PushableByExplosion;

fn start_explosions(
    mut reader: EventReader<StartExplosion>,
    mut commands: Commands,
    mut pbr: CachedPbrMaker,
) {
    for event in reader.read() {
        let mut cmd = commands.spawn(pbr.make_pbr_with(
            || {
                Mesh::try_from(shape::Icosphere {
                    radius: 1.0,
                    subdivisions: 10,
                })
                .unwrap()
            },
            // || Color::RED.into(),
            || StandardMaterial {
                base_color: Color::Rgba {
                    red: 1.0,
                    green: 0.3,
                    blue: 0.3,
                    alpha: 0.05,
                },
                // base_color_texture: todo!(),
                emissive: Color::Rgba {
                    red: 1.0,
                    green: 0.5,
                    blue: 0.5,
                    alpha: 0.1,
                },
                // emissive_texture: todo!(),
                // perceptual_roughness: todo!(),
                // metallic: todo!(),
                // metallic_roughness_texture: todo!(),
                // reflectance: todo!(),
                // diffuse_transmission: todo!(),
                // specular_transmission: 0.8,
                thickness: 1.0,
                ior: 1.0,
                // attenuation_distance: todo!(),
                // attenuation_color: todo!(),
                // normal_map_texture: todo!(),
                // flip_normal_map_y: todo!(),
                // occlusion_texture: todo!(),
                // double_sided: todo!(),
                // cull_mode: todo!(),
                // unlit: todo!(),
                fog_enabled: true,
                alpha_mode: AlphaMode::Blend,
                // depth_bias: todo!(),
                // depth_map: todo!(),
                // parallax_depth_scale: todo!(),
                // parallax_mapping_method: todo!(),
                // max_parallax_layer_count: todo!(),
                // opaque_render_method: todo!(),
                // deferred_lighting_pass_id: todo!(),
                ..Default::default()
            },
        ));
        cmd.insert(Transform::from_translation(event.position.extend(0.0)).with_scale(Vec3::ZERO));
        cmd.insert(YoleckBelongsToLevel { level: event.level });

        cmd.insert(Collider::ball(1.0));
        cmd.insert(Sensor);

        cmd.insert(ExplosionStatus {
            timer: Timer::from_seconds(0.30, TimerMode::Once),
        });
    }
}

fn progress_explosion_lifetime(
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionStatus, &mut Transform)>,
    mut commands: Commands,
) {
    for (entity, mut status, mut transform) in query.iter_mut() {
        if status.timer.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            let progress = status.timer.elapsed_secs() / status.timer.duration().as_secs_f32();
            transform.scale = 6.0 * progress.powf(0.125) * Vec3::ONE;
        }
    }
}

fn apply_explosion_force(
    time: Res<Time>,
    explosions_query: Query<(Entity, &GlobalTransform), With<ExplosionStatus>>,
    rapier_context: Res<RapierContext>,
    mut pushables_query: Query<(&GlobalTransform, &mut Velocity), With<PushableByExplosion>>,
) {
    for (explosion_entity, explosion_transform) in explosions_query.iter() {
        for (e1, e2, intersecting) in rapier_context.intersections_with(explosion_entity) {
            if !intersecting {
                continue;
            }
            let pushable_entity = if e1 == explosion_entity { e2 } else { e1 };
            let Ok((pushable_transform, mut pushable_velocity)) =
                pushables_query.get_mut(pushable_entity)
            else {
                continue;
            };
            let push_vector =
                (pushable_transform.translation() - explosion_transform.translation()).truncate();
            if push_vector.length_squared() == 0.0 {
                continue;
            }
            let force_vector =
                1000.0 * (push_vector / push_vector.length_squared()).clamp_length_max(10.0);
            pushable_velocity.linvel += time.delta_seconds() * force_vector;
        }
    }
}
