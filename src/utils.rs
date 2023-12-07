use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn collision_started_events_both_ways<'a>(
    reader: &'a mut EventReader<CollisionEvent>,
) -> impl 'a + Iterator<Item = (Entity, Entity)> {
    reader
        .read()
        .filter_map(|event| {
            if let CollisionEvent::Started(e1, e2, _) = event {
                Some([(*e1, *e2), (*e2, *e1)])
            } else {
                None
            }
        })
        .flatten()
}

#[derive(SystemParam)]
pub struct CachedPbrMaker<'w, 's> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    mesh_and_material: Local<'s, Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
}

impl CachedPbrMaker<'_, '_> {
    pub fn make_pbr_with(
        &mut self,
        mesh: impl FnOnce() -> Mesh,
        material: impl FnOnce() -> StandardMaterial,
    ) -> PbrBundle {
        let (mesh, material) = self
            .mesh_and_material
            .get_or_insert_with(|| (self.meshes.add(mesh()), self.materials.add(material())))
            .clone();
        PbrBundle {
            mesh,
            material,
            ..Default::default()
        }
    }
}
