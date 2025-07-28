use std::collections::BTreeMap;

use bevy::{prelude::*, utils::HashMap};

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<StatChangeEvent>();
    }
}




#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Reflect, Hash)]
pub enum StatType {
    Health,
    Breath,
}

#[derive(Clone, Reflect)]
pub struct Stat {
    pub base: f32,
    pub effective: f32,
    pub min: f32,
    pub max: f32,
}
impl Stat {
    pub fn new(min: f32, max: f32) -> Stat {
        Stat {base: max, effective: max, min, max}
    }

    pub fn with_value(value: f32, min: f32, max: f32) -> Stat {
        Stat {base: value, effective: value, min, max}
    }
}

// Components
#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct Stats(pub HashMap<StatType, Stat>);
impl Stats {
    pub fn get_base (&self, stat: &StatType) -> f32 {
        // TODO: Check if we have the requested value. Otherwise, give 0 and print an error or something.
        if self.0.contains_key(stat) {
            self.0[stat].base
        } else {
            eprintln!("ERROR: Stat not found! Returning zero.");
            0.0
        }
    }

    pub fn get_effective (&self, stat: &StatType) -> f32 {
        // TODO: Check if we have the requested value. Otherwise, give 0 and print an error or something.
        if self.0.contains_key(stat) {
            self.0[stat].effective
        } else {
            eprintln!("ERROR: Stat not found! Returning zero.");
            0.0
        }
    }

    /*
    pub fn get_mut_value (&mut self, stat: &String) -> &mut i32 {
        &mut self.0.get_mut(stat).unwrap().value
    }
     */

    pub fn get_min (&self, stat: &StatType) -> f32 {
        self.0[stat].min
    }

    pub fn get_max (&self, stat: &StatType) -> f32 {
        self.0[stat].max
    }

    pub fn in_range (&self, stat: &StatType, value: f32) -> bool {
        value <= self.0[stat].max && value >= self.0[stat].min
    }
}

//Events
#[derive(Clone, Copy, Event)]
pub struct StatChangeEvent {
    pub stat: StatType,
    pub amount: f32,
    pub entity: Entity,
}
impl StatChangeEvent {
    pub fn new(stat: StatType, amount: f32, entity: Entity) -> Self {
        StatChangeEvent {stat, amount, entity}
    }
}

//Systems
pub fn do_stat_change (
    mut evr_stat_change: EventReader<StatChangeEvent>,

    mut stats_query: Query<&mut Stats>,
) {
    for ev in evr_stat_change.read() {
        if let Ok(mut stats) = stats_query.get_mut(ev.entity) {
            //println!("stat type: {}", ev.stat);
            stats.get_mut(&ev.stat).unwrap().base += ev.amount;
            
            stats.get_mut(&ev.stat).unwrap().base = stats.get_base(&ev.stat).clamp(stats.get_min(&ev.stat), stats.get_max(&ev.stat));
        }
    }
}