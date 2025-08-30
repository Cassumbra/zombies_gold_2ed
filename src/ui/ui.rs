use std::cmp::min;

use bevy::{a11y::AccessibilityNode, prelude::*};
use iyes_perf_ui::PerfUiCompleteBundle;

use crate::{hotbar::{Hotbar, SlotAction}, Atlas, BuildingEvent, BuildingTimer, HasAir, Inventory, ItemID, MiningEvent, MiningTimer, Player, StatChangeEvent, StatType, Stats};


pub fn setup_ui (
    mut commands: Commands,
    //asset_server: Res<AssetServer>,
    atlas: Res<Atlas>,
) {
    commands.spawn(PerfUiCompleteBundle::default());

    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,

            ..default()
        },

    ..default()
    }).with_children(|parent| {
        // The stupid thingy that keeps the crosshair centered.
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(50.0),
                height: Val::Px(10.0),
                ..default()
            },
            //background_color: BackgroundColor(Color::WHITE),
            ..default()
        });

        // Crosshair
        parent.spawn((
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "+",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.25 },
                    ..default()
                },
            ) // Set the justification of the Text
            .with_text_justify(JustifyText::Center)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                ..default()
            }),
        ));

        // Progress Bar
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(50.0),
                height: Val::Px(10.0),
                ..default()
            },
            background_color: BackgroundColor(Color::Rgba { red: 0.9, green: 0.9, blue: 0.9, alpha: 0.25 },),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.50 },),
                ..default()
            })
            .insert(ProgressBar::None);
        });
    });

    

    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            //justify_content: JustifyContent::Start,
            //align_items: AlignItems::Start,

            ..default()
        },

        ..default()
    })
    .insert(ItemDisplayRoot);

    /*
    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            
            ..default()
        },

        ..default()
    });
    //.insert(HotBarRoot); */

    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            align_self: AlignSelf::End,
            justify_self: JustifySelf::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            
            ..default()
        },

    ..default()
    }).with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                
                //position_type: PositionType::Absolute,
                //align_items: AlignItems::Center,
                //justify_items: JustifyItems::Center,
                align_self: AlignSelf::Start,
                justify_self: JustifySelf::Start,
                align_content: AlignContent::Start,
                justify_content: JustifyContent::Start,
                
                ..default()
            },

            ..default()
        }).insert(HealthBarRoot).with_children(|parent| {
            for _ in 0..=3 {
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(64.),
                        height: Val::Px(64.),
                        ..default()
                    },
                    image: UiImage::new(atlas.ui_16x16.clone()),
                    ..default()
                    },
                    )
                    .insert(TextureAtlas{ layout: atlas.ui_16x16_layout.clone(), index: 19 as usize})
                    .insert(HealthDisplay);
            }
        });

        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                
                /*
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                align_self: AlignSelf::End,
                justify_self: JustifySelf::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center, */
                
                ..default()
            },

            ..default()
        }).insert(HotBarRoot).with_children(|parent| {
            for _ in 0..=9 {
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(64.),
                        height: Val::Px(64.),
                        ..default()
                    },
                    image: UiImage::new(atlas.ui_16x16.clone()),
                    ..default()
                    },
                    )
                    .insert(TextureAtlas{ layout: atlas.ui_16x16_layout.clone(), index: 34 as usize})
                    .insert(HotBarSlot);
            }
        });
        
        }
        );

        commands.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                
                position_type: PositionType::Absolute,
                align_items: AlignItems::End,
                justify_items: JustifyItems::End,
                align_self: AlignSelf::End,
                justify_self: JustifySelf::End,
                align_content: AlignContent::End,
                justify_content: JustifyContent::End,

                margin: UiRect::all(Val::Percent(0.0)),
                padding:UiRect::all(Val::Percent(0.0)),
                
                ..default()
            },
    
        ..default()
        })
        .insert(BreathRoot)
        .with_children(|parent| {
                parent.spawn(TextBundle::from_section("100", TextStyle { font_size: 75.0, color: Color::WHITE, ..default()}))
                    .insert(BreathValue);

                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(64.),
                        height: Val::Px(64.),
                        ..default()
                    },
                    image: UiImage::new(atlas.ui_16x16.clone()),
                    ..default()
                    },
                    )
                    .insert(TextureAtlas{ layout: atlas.ui_16x16_layout.clone(), index: 3 as usize});
            });
}

pub fn update_breath_ui (
    mut commands: Commands,

    player_query: Query<(&mut Stats), With<Player>>,
    mut root_query: Query<(&mut Style), With<BreathRoot>>,
    mut breath_value_query: Query<(&mut Text), With<BreathValue>>,
) {
    let stats = player_query.get_single().unwrap();
    let mut breath_root_style = root_query.get_single_mut().unwrap();
    let mut breath_value_text = breath_value_query.get_single_mut().unwrap();

    if stats[&StatType::Breath].base < 100.0 {
        breath_root_style.display = Display::Flex;
    }
    else {
        breath_root_style.display = Display::None;
    }
    *breath_value_text = Text::from_section((stats[&StatType::Breath].base as i32).to_string(), TextStyle { font_size: 75.0, color: Color::WHITE, ..default()});
}

pub fn update_resource_counts (
    mut commands: Commands,

    inventory_query: Query<(&Inventory), (With<Player>, Changed<Inventory>)>,
    root_query: Query<(Entity), With<ItemDisplayRoot>>,
    item_display_query: Query<(Entity), With<ItemDisplay>>,

    atlas: Res<Atlas>,
) {
    if let Ok(inventory) = inventory_query.get_single() {
        if let Ok(root) = root_query.get_single() {
            for (entity) in &item_display_query {
                commands.entity(entity).despawn_recursive();
            }

            let mut opt_last_item_id: Option<ItemID> = None;
    
            for item in inventory.iter() {
                if Some(item.id) == opt_last_item_id {
                    continue;
                }
    
                
                let number_entity = commands.spawn(TextBundle::from_section(inventory.get_item_amount(item.id).to_string(), TextStyle { font_size: 100.0, color: Color::WHITE, ..default()}))
                    .id();

                let tex_coords = item.get_tex_coords();
                let index = tex_coords.x + tex_coords.y * 32;

                //println!("coords: {}, index: {}", tex_coords, index);

                let image_entity = commands.spawn(ImageBundle {
                    style: Style {
                    //    width: Val::Px(256.),
                    //    height: Val::Px(256.),
                        ..default()
                    },
                    image: UiImage::new(atlas.items_8x8.clone()),
                    ..default()
                    },
                    )
                    .insert(TextureAtlas{ layout: atlas.items_8x8_layout.clone(), index: index as usize})
                    .id();
                

                let display_entity = commands.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,

                        ..default()
                    },

                    ..default()
                })
                .insert(ItemDisplay(item.id))
                .add_child(number_entity)
                .add_child(image_entity)
                .id();
    
                commands.entity(root).add_child(display_entity);
    
                opt_last_item_id = Some(item.id);
            }
        }
    }
}

pub fn update_health_bar (
    mut commands: Commands,

    player_query: Query<&Stats, (With<Player>, Or<(Changed<Stats>, Added<Stats>)>)>,

    //inventory_query: Query<(&Inventory), (With<Player>, Changed<Inventory>)>,
    root_query: Query<(Entity), With<HealthBarRoot>>,
    health_display_query: Query<(Entity), With<HealthDisplay>>,

    atlas: Res<Atlas>,
) {
    if let Ok(stats) = player_query.get_single() {
        if let Ok(root) = root_query.get_single() {
            for entity in &health_display_query {
                commands.entity(entity).despawn_recursive();
            }
            let mut remaining_health = stats.get_base(&StatType::Health);
            for _ in 0..(stats.get_max(&StatType::Health)/4.0).ceil() as i32 {
                // TODO: Make this some sort of constant
                let base_index = 19;
                let index = base_index + (4 - min(remaining_health as usize, 4));

                let health_display_unit = commands.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(64.),
                            height: Val::Px(64.),
                            ..default()
                        },
                        image: UiImage::new(atlas.ui_16x16.clone()),
                        ..default()
                        },
                        )
                    .insert(TextureAtlas{ layout: atlas.ui_16x16_layout.clone(), index})
                    .insert(HealthDisplay)
                    .id();

                commands.entity(root).add_child(health_display_unit);

                remaining_health -= 4.0;
                if remaining_health < 0.0 {remaining_health = 0.0};
            }
        }
    }
}

pub fn update_hotbar (
    mut commands: Commands,

    player_query: Query<&Hotbar, (With<Player>, Or<(Changed<Hotbar>, Added<Hotbar>)>)>,

    //inventory_query: Query<(&Inventory), (With<Player>, Changed<Inventory>)>,
    root_query: Query<(Entity), With<HotBarRoot>>,
    hotbar_slot_query: Query<(Entity), With<HotBarSlot>>,

    atlas: Res<Atlas>,
) {
    if let Ok(hotbar) = player_query.get_single() {
        if let Ok(root) = root_query.get_single() {
            for entity in &hotbar_slot_query {
                commands.entity(entity).despawn_recursive();
            }
            for i in 0..hotbar.slots.len() {
                // TODO: Make this some sort of constant
                let index = if hotbar.position == i {35} else {34};

                let hotbar_slot = commands.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(64.),
                            height: Val::Px(64.),

                            //flex_direction: FlexDirection::Column,
                            //position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center,
                            //align_self: AlignSelf::Center,
                            //justify_self: JustifySelf::Center,
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,

                            ..default()
                        },
                        image: UiImage::new(atlas.ui_16x16.clone()),
                        ..default()
                        },
                        )
                    .insert(TextureAtlas{ layout: atlas.ui_16x16_layout.clone(), index})
                    .insert(HotBarSlot)
                    .id();

                if hotbar.slots[i] != SlotAction::None {
                    let action_icon = commands.spawn(ImageBundle {
                        style: Style {
                            width: Val::Px(32.),
                            height: Val::Px(32.),

                            flex_direction: FlexDirection::Column,
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center,
                            align_self: AlignSelf::Center,
                            justify_self: JustifySelf::Center,
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,

                            ..default()
                        },
                        image: UiImage::new(atlas.res_8x8.clone()),
                        ..default()
                        },
                        )
                    .insert(TextureAtlas{ layout: atlas.res_8x8_layout.clone(), index: match hotbar.slots[i] {
                        SlotAction::None => {panic!()},
                        SlotAction::Block(block_id) => {
                            let coords = block_id.get_attributes().tex_coords.top;
                            // TODO: Hardcoded values are cringe.
                            (coords.y * 32 + coords.x) as usize
                        },
                        SlotAction::Item(_) => todo!(),
                    }})
                    .insert(HotBarSlot)
                    .id();
                    // TODO: Display the number of blocks we can place given our current resources
                    commands.entity(hotbar_slot).add_child(action_icon);
                }
                

                

                commands.entity(root).add_child(hotbar_slot);
            }
        }
    }
}


const PROGRESS_BAR_SMOOTHNESS: f32 = 12.0;

pub fn update_progress_bar (

    mut bar_query: Query<(&mut Style, &mut ProgressBar)>,
    timers_query: Query<(&BuildingTimer, &MiningTimer)>,

    evr_building: EventReader<BuildingEvent>,
    evr_mining: EventReader<MiningEvent>,
) {
    if let Ok((mut style, mut progress_bar)) = bar_query.get_single_mut() {
        if let Ok((building_timer, mining_timer)) = timers_query.get_single() {
            if !evr_building.is_empty() {
                *progress_bar = ProgressBar::Building;
            }
    
            if !evr_mining.is_empty() {
                *progress_bar = ProgressBar::Mining;
            }
    
            let mut timer_fraction = 0.0;

            match *progress_bar {
                ProgressBar::None => {},
                ProgressBar::Building =>  {
                    timer_fraction = building_timer.fraction()
                },
                ProgressBar::Mining => {
                    timer_fraction = mining_timer.fraction()
                },
                
            }

            style.width = Val::Percent(((timer_fraction * PROGRESS_BAR_SMOOTHNESS).round() / PROGRESS_BAR_SMOOTHNESS) * 100.0);
            //style.width = Val::Percent(timer_fraction * 100.0);
        }
        
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum ProgressBar {
    None,
    Building,
    Mining,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ItemDisplayRoot;

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ItemDisplay(ItemID);

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct HotBarRoot;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct HotBarSlot;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct HealthBarRoot;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct HealthDisplay;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct BreathRoot;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct BreathValue;