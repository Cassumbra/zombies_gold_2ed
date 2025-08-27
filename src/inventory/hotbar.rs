use bevy::prelude::*;

use crate::BlockID;

// Components
#[derive(Clone, Debug, Component)]
pub struct Hotbar {
    pub position: u8,
    pub slots: Vec<SlotAction>,
}
impl Default for Hotbar {
    fn default() -> Self {
        Self { position: 0, slots: vec![SlotAction::Block(BlockID::Planks), SlotAction::Block(BlockID::StoneBrick), SlotAction::None, 
                                        SlotAction::None, SlotAction::None, SlotAction::None, 
                                        SlotAction::None, SlotAction::None, SlotAction::None, 
                                        SlotAction::None, ]}
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotAction {
    #[default] None,
    Block(BlockID)
}