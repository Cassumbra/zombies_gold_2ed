use bevy::prelude::*;

use movement::*;
pub mod movement;


pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>();
    }
}