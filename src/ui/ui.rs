use bevy::{a11y::AccessibilityNode, prelude::*};

use crate::{Atlas, Inventory, ItemID, Player};

pub fn setup_ui (
    mut commands: Commands,
    //asset_server: Res<AssetServer>,
) {
    commands.spawn((
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
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            ..default()
        }),
    ));

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

                println!("coords: {}, index: {}", tex_coords, index);

                let image_entity = commands.spawn(ImageBundle {
                    //style: Style {
                    //    width: Val::Px(256.),
                    //    height: Val::Px(256.),
                    //    ..default()
                    //},
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

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ItemDisplayRoot;

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ItemDisplay(ItemID);