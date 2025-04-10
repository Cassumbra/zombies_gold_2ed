use std::collections::BTreeMap;

use bevy::{prelude::*, utils::HashMap};


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
#[derive(Component, Default, Clone, Reflect, Deref, DerefMut)]
#[reflect(Component)]
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