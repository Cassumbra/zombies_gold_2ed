// Alt names for this crate: Biology. Effects.
// Went with Mechanics though as we may put some processes here that are not strictly biological.
// Crate was created with the intent of handling suffocating and likely later things in a similar vein.

use std::cmp::max;

use bevy::prelude::*;

use crate::{StatType, Stats};

//Components
#[derive(Component, Clone, Copy, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct HasAir(pub bool);

pub fn handle_breath (
    mut commands: Commands,

    mut query: Query<(&mut Stats, &HasAir)>,

    time: Res<Time>,

) {
    for (mut stats, has_air) in &mut query {
        if stats.contains_key(&StatType::Breath) {
            if **has_air {
                stats.get_mut(&StatType::Breath).unwrap().base += time.delta_seconds() * 10.0;
                if stats[&StatType::Breath].base > stats[&StatType::Breath].max {
                    stats.get_mut(&StatType::Breath).unwrap().base = stats[&StatType::Breath].max;
                }
            }
            else {
                stats.get_mut(&StatType::Breath).unwrap().base -= time.delta_seconds() * 2.5;
                if stats[&StatType::Breath].base < stats[&StatType::Breath].min {
                    stats.get_mut(&StatType::Breath).unwrap().base = stats[&StatType::Breath].min;
                }
            }
            //println!("{}", stats[&StatType::Breath].base);
        }
    }
}