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
