// Alt names for this crate: Biology. Effects.
// Went with Mechanics though as we may put some processes here that are not strictly biological.
// Crate was created with the intent of handling suffocating and likely later things in a similar vein.

use core::f32;
use std::cmp::max;

use bevy::prelude::*;

use crate::{FallEvent, MoveToSpawn, StatChangeEvent, StatType, Stats};


pub struct MechanicsPlugin;

impl Plugin for MechanicsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<DeathEvent>();
    }
}

//Events
#[derive(Clone, Copy, Event)]
pub struct DeathEvent {
    pub perpetrator: Instigator,
    pub cause: EffectCause,
    pub entity: Entity,
}
impl DeathEvent {
    pub fn new(perpetrator: Instigator, cause: EffectCause, entity: Entity) -> Self {
        DeathEvent {perpetrator, cause, entity}
    }
}

//Components
#[derive(Component, Clone, Copy, Deref, DerefMut)]
pub struct HasAir(pub bool);

//#[derive(Component, Clone, Deref, DerefMut)]
//pub struct InvincibilityTimer(Timer);

//Data
#[derive(Clone, Copy, Debug)]
pub enum Instigator {
    World,
    Entity(Entity),
}

#[derive(Clone, Copy, Debug)]
pub enum EffectCause {
    Drowning,
    Fall,
    Revive,
}

pub fn handle_breath (
    query: Query<(&Stats, &HasAir, Entity)>,

    time: Res<Time>,

    mut evw_stat_change: EventWriter<StatChangeEvent>,
) {
    for (stats, has_air, entity) in &query {
        if stats.contains_key(&StatType::Breath) {
            if **has_air {
                evw_stat_change.send(StatChangeEvent::new(Instigator::World, EffectCause::Drowning, StatType::Breath, time.delta_seconds() * 30.0, entity));
            }
            else {
                evw_stat_change.send(StatChangeEvent::new(Instigator::World, EffectCause::Drowning, StatType::Breath, time.delta_seconds() * -6.5, entity));
            }
        }
    }
}

pub fn handle_suffocation (
    query: Query<(&Stats, &HasAir, Entity)>,

    mut evw_death: EventWriter<DeathEvent>,

) {
    for (stats, has_air, entity) in &query {
        if stats.contains_key(&StatType::Breath) { //&& stats.contains_key(&StatType::Health)
            if !**has_air && stats.get(&StatType::Breath).unwrap().base <= 0.0 {
                // Just kill the sucker. We might do gradual HP loss later if that seems more appropriate.
                evw_death.send(DeathEvent::new(Instigator::World, EffectCause::Drowning, entity));
            }
        }
    } 
}

pub fn handle_fall_damage (
    mut evr_fall: EventReader<FallEvent>,
    mut evw_stat_change: EventWriter<StatChangeEvent>,
) {
    for ev in evr_fall.read() {
        if ev.distance <= -4.0 {
            evw_stat_change.send(StatChangeEvent::new(Instigator::World, EffectCause::Fall, StatType::Health, (ev.distance + 3.0).floor(), ev.entity));
        }
    }
}

pub fn handle_death (
    mut commands: Commands,

    query: Query<(&Stats)>,

    mut evr_death: EventReader<DeathEvent>,
    mut evw_stat_change: EventWriter<StatChangeEvent>,
) {
    for ev in evr_death.read() {
        // TODO: Handle dropping of the players resources here
        // TODO: Add some kind of sound effect for dying.
        // TODO: Perhaps some sort of respawn dialogue before respawning the player? Unsure. Perhaps some timer...
        if let Ok(stats) = query.get(ev.entity) {
            if stats.contains_key(&StatType::Health) {
                evw_stat_change.send(StatChangeEvent::new(Instigator::World, EffectCause::Revive, StatType::Health, f32::MAX, ev.entity));
            }
            if stats.contains_key(&StatType::Breath) {
                evw_stat_change.send(StatChangeEvent::new(Instigator::World, EffectCause::Revive, StatType::Breath, f32::MAX, ev.entity));
            }
        }
        commands.entity(ev.entity).insert(MoveToSpawn);
    }
}