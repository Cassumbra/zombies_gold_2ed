use std::{collections::VecDeque, fmt::Display};

use hotbar::*;
use bevy::{prelude::*, utils::hashbrown::HashMap};
use itertools::Itertools;

pub mod hotbar;

const MAX_MATERIAL: u16 = 2048;
const INVENTORY_SIZE: IVec2 = IVec2::new(9, 2);


// TODO: We might want to make this a non tuple struct later on when we have non player inventories that differ in size from the player's inventory and will need a unique inventory_size field.
#[derive(Clone, Component, Deref, DerefMut)]
pub struct Inventory(pub Vec<Item>);
impl Default for Inventory {
    fn default() -> Self {
        Self(Vec::with_capacity((INVENTORY_SIZE.x * INVENTORY_SIZE.y) as usize))
    }
}
impl Inventory {
    pub fn consolidate (&mut self) {
        let mut item_totals = HashMap::<ItemID, u16>::new();

        for slot in self.iter() {
            if let Some(total) = item_totals.get_mut(&slot.id) {
                *total += slot.amount;
            }
            else {
                item_totals.insert(slot.id, slot.amount);
            }
        }

        let mut item_totals: Vec<Item> = item_totals.iter().map(|(id, total)| Item{ id: *id, amount: *total }).collect();
        item_totals.sort_unstable_by_key(|item| item.id);

        self.clear();
        
        for item in item_totals.iter_mut() {
            let attributes = item.id.get_attributes();
            while item.amount > 0 {
                if item.amount < attributes.max_amount {
                    self.push(Item { id: item.id, amount: item.amount });
                    item.amount = 0;
                } else {
                    self.push(Item { id: item.id, amount: attributes.max_amount});
                    item.amount -= attributes.max_amount;
                }
                
            }
        }
    }

    pub fn get_item_amount(&self, item_id: ItemID) -> u16 {
        let mut total = 0;
        for slot in self.iter() {
            if slot.id == item_id {
                total += slot.amount;
            }
        }

        total
    }

    // These two functions assume that our inventory is already consolidated. Also, we do not allow partial inserts. Should we?
    pub fn insert_item(&mut self, item: Item) -> Result<u16, ItemInsertFault> {
        let mut result = Result::Err(ItemInsertFault::NoSpace);

        for slot in self.iter_mut() {
            let attributes = slot.get_attributes();
            if slot.id == item.id && slot.amount + item.amount < attributes.max_amount {
                slot.amount += item.amount;
                result = Result::Ok(slot.amount);
                break;
            }
        }

        if result.is_err() &&self.len() < (INVENTORY_SIZE.x * INVENTORY_SIZE.y) as usize {
            self.push(item);
            result = Result::Ok(item.amount);
        }

        self.consolidate();

        return result;
    }

    pub fn take_item(&mut self, item: Item) -> Result<Item, ItemExtractFault> {
        let mut result = Err(ItemExtractFault::NoItem);

        for (i, slot) in self.iter_mut().enumerate() {
            if slot.id == item.id {
                    if slot.amount >= item.amount {
                        if slot.amount == item.amount {
                        result = Ok(self.remove(i));
                        break;
                    }
                    else {
                        slot.amount -= item.amount;
                        result = Ok(item);
                        break;
                    }
                    
                }
                else {
                    result = Err(ItemExtractFault::InsufficientAmount);
                    break;
                }
                
            }
        }

        self.consolidate();

        return result;
    }
}

// TODO: Is it even helpful for us to have these different kinds of faults? Can we just return () or something in the Errs? Might make things simpler.
#[derive(Clone, Copy)]
pub enum ItemInsertFault {
    NoSpace,
    InsufficientSpace,
}

#[derive(Clone, Copy)]
pub enum ItemExtractFault {
    NoItem,
    InsufficientAmount
}

#[derive(Clone, Copy,)]
pub struct Item {
    pub id: ItemID,
    pub amount: u16,
}
impl Item {
    pub fn new(id: ItemID, amount: u16) -> Item {
        // TODO: Make the ItemData thing be tailored for the Item we're making.
        Item {id, amount } //data: [ItemData::None]}
    }

    pub fn get_attributes(self) -> ItemAttributes {
        self.id.get_attributes()
    }

    pub fn get_tex_coords(self) -> IVec2 {
        let attributes = self.id.get_attributes();
        let coords = attributes.tex_coords;
        IVec2::new(coords.x + (self.amount.min(attributes.max_amount) / attributes.coord_increment_num) as i32, coords.y)
    }
}


#[derive(Default, Clone, Copy)]
pub struct ItemAttributes {
    pub tex_coords: IVec2,
    /// The amounts we need to go up by to raise the coords' x value by 1. 0 or lower disables this.
    pub coord_increment_num: u16,
    // The number where we stop incrementing the coords' x value
    //pub coord_increment_limit: u16,
    pub max_amount: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect, Debug)]
pub enum ItemID {
    //BuildingMaterial(BuildingMaterial),
    Stone,
    Wood,
}
impl ItemID {
    fn get_attributes(self) -> ItemAttributes {
        match self {
            ItemID::Stone => ItemAttributes { tex_coords: IVec2::new(0, 0), coord_increment_num: MAX_MATERIAL / 4, max_amount: MAX_MATERIAL}, //coord_increment_limit: MAX_MATERIAL },
            ItemID::Wood => ItemAttributes { tex_coords: IVec2::new(0, 1), coord_increment_num: MAX_MATERIAL / 4, max_amount: MAX_MATERIAL},
        }
    }
}
impl Display for ItemID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemID::Stone => write!(f, "Stone"),
            ItemID::Wood => write!(f, "Wood"),
        }
    }
}

/*
#[derive(Clone, Copy)]
pub enum BuildingMaterial {
    Stone,
    Wood,
}
 */