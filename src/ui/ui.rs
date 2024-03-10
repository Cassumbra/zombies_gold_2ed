use bevy::prelude::*;

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
}