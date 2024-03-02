#![cfg_attr(
    all(
      target_os = "windows",
      not(feature = "console"),
    ),
    windows_subsystem = "windows"
)]

#![feature(const_fn_floating_point_arithmetic)]

use bevy::{app::AppExit, ecs::schedule::ScheduleLabel, pbr::wireframe::WireframePlugin, prelude::*, render::{settings::{RenderCreation, WgpuFeatures}, RenderPlugin}, window::{exit_on_all_closed, exit_on_primary_closed}};
use bevy_asset_loader::prelude::*;
//use bevy_flycam::PlayerPlugin;
use bevy_xpbd_3d::{math::{Scalar, Vector}, prelude::*};
use leafwing_input_manager::prelude::*;
use moonshine_save::{save::SavePlugin, load::LoadPlugin};
use bevy::render::settings::WgpuSettings;

//use sark_grids::Grid;

#[path = "spatial/spatial.rs"]
mod spatial;
use spatial::*;


#[path = "actions/actions.rs"]
mod actions;
use actions::*;

/*
#[path = "log/log.rs"]
mod log;
use log::*;
 */

#[path = "map/map.rs"]
mod map;
use map::*;


#[path = "player/player.rs"]
mod player;
use player::*;

#[path = "rendering/rendering.rs"]
mod rendering;
use rendering::*;

/*
#[path = "saveload/saveload.rs"]
mod saveload;
use saveload::*;

#[path = "setup/setup.rs"]
mod setup;
use setup::*;

#[path = "simulation/simulation.rs"]
mod simulation;
use simulation::*;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, ScheduleLabel, Default)]
pub enum GameState {
    #[default] Setup, LoadOrNew, PostLoad, PickName, MapGen, SpawnActors, //FinishSetup,
    Playing, Targetting,
    Restart,
    //Save,
    //SaveQuit,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, ScheduleLabel, Default)]
pub enum MapGenState {
    #[default] LargeLandmasses, TempBand, Precipitation, Biomes, Snow, Rivers, CleanRivers, InitialSites, Finished,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, ScheduleLabel, Default)]
pub enum PlayingState {
    #[default] BigMap,
}
 */

 #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, ScheduleLabel, Default)]
pub enum GameState {
    #[default] AssetLoading,
    //Setup,
    Playing,
}

fn main () {
    App::new()
    //.insert_resource(WgpuOptions {
    //    features: WgpuFeatures::POLYGON_MODE_LINE,
    //    ..Default::default()
    //})
    .add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3D Zombies Gold".into(),
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest())
    )
    .add_plugins(WireframePlugin)
    .add_plugins(PhysicsPlugins::default())
    //.insert_resource(Msaa::Sample4)

    .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Playing)
                .load_collection::<Atlas>(),
        )
    /*
    .add_plugins(DefaultPlugins.set(RenderPlugin {
        render_creation: RenderCreation::Automatic(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        },
    ),
    ..default()
    }))
      */

    // This plugin maps inputs to an input-type agnostic action-state
    // We need to provide it with an enum which stores the possible actions a player could take
    .add_plugins(InputManagerPlugin::<Action>::default())
    // The InputMap and ActionState components will be added to any entity with the Player component

    //.add_plugins(PlayerPlugin)

    .add_plugins(ActionsPlugin)

    .init_resource::<RNGSeed>()

    .add_systems(OnEnter(GameState::Playing), map::generate_small_map)
    .add_systems(Startup, setup)

    .add_systems(Update, rendering::update_chunk_meshes.run_if(in_state(GameState::Playing)))
    .add_systems(Update, update_chunk_colliders)
    
    .add_systems(
        Update,
        (
            player_input_game,
            movement::update_grounded,
            movement::movement,
            movement::apply_movement_damping,
        )
            .chain(),
    )
    
    /*
    .add_systems(PostUpdate, save_game()
        .include_resource::<animated_tiles::ScrollingNoiseSpeed>()
        .into_file(SAVE_PATH)
            .after(exit_on_primary_closed)
            .after(exit_on_all_closed)
            .run_if(app_exit))
    */

    .run();
}

#[derive(AssetCollection, Resource)]
struct Atlas{
    #[asset(path = "textures_8x8.png")]
    pub res_8x8: Handle<Image>,
}

#[derive(Clone, Copy, Resource, Deref, DerefMut, Reflect)]
pub struct RNGSeed(u32);
impl Default for RNGSeed {
    fn default() -> Self {
        Self(2343)
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    MoveForward, MoveBackward,
    MoveLeft, MoveRight,
    Crouch, Jump,
    Look, Primary,
    MenuBack,
}

const INPUT_MAP: [(Action, InputKind); 9] = [(Action::MoveForward, InputKind::PhysicalKey(KeyCode::KeyW)), (Action::MoveBackward, InputKind::PhysicalKey(KeyCode::KeyS)),
                                            (Action::MoveLeft, InputKind::PhysicalKey(KeyCode::KeyA)), (Action::MoveRight, InputKind::PhysicalKey(KeyCode::KeyD)),
                                            (Action::Crouch, InputKind::PhysicalKey(KeyCode::ShiftLeft)), (Action::Jump, InputKind::PhysicalKey(KeyCode::Space)),
                                            (Action::Look, InputKind::DualAxis(DualAxis::mouse_motion())), (Action::Primary, InputKind::Mouse(MouseButton::Left)),
                                            (Action::MenuBack, InputKind::PhysicalKey(KeyCode::Escape)),
                                          ];

pub fn app_exit (mut events: EventReader<AppExit>) -> bool {
    !events.is_empty()
}


pub fn update_chunk_colliders (
    mut commands: Commands,

    query: Query<(Entity, &Chunk), Or<(Added<Chunk>, Changed<Chunk>)>>,
) {
    for (entity, chunk) in &query {
        // TODO: Optimize this. we don't need colliders if a block is touching air.
        let colliders: Vec::<(Vector, Quat, Collider)> = chunk.iter_3d().filter_map(|(position, block)| {
            if block.block_id != BlockID::Air {
                Some((Vector::from(position.as_vec3() + Vec3::new(1.5, 1.5, 1.5)), Quat::IDENTITY, Collider::cuboid(1.0, 1.0, 1.0)))
            }
            else {
                None
            }
            
        
        }).collect();

        commands.entity(entity)
        .insert(Collider::compound(colliders))
        .insert(RigidBody::Static);
    }
}



pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {

    let height = 1.0;

    // Emotional support cube. Uncomment when needed.
    /*
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(7.0, 25.0, 7.0),
            ..default()
        },
    ))
    .insert(RigidBody::Dynamic)
    .insert(Collider::cuboid(1.0, 1.0, 1.0));
     */
    
    // Camera
    let camera = commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, height * 0.8, 0.0),
        ..default()
    })
    .id();

    // Player
    commands.spawn((
        /*
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.4, height)),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 1.5, 0.0),
            ..default()
        },
         */
        SpatialBundle::default(),
        movement::CharacterControllerBundle::new(Collider::cylinder(height, 0.4)).with_movement(
            30.0,
            0.92,
            7.0,
            (30.0 as Scalar).to_radians(),
        ),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        GravityScale(2.0),
        Player,
        InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new(INPUT_MAP),
        },
        //Transform::default(),
        //GlobalTransform::default(),
    ))
    .add_child(camera);
    


    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2_000_000.0,
            range: 50.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 15.0, 0.0),
        ..default()
    });

    
}

/*
pub fn add_input_manager (
    mut commands: Commands,

    query: Query<(Entity), (With<Player>)>,
) {
    let (player) = query.single();

    commands.entity(player)
        .insert(InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new(INPUT_MAP),
        });
    

    commands.insert_resource(NextState(Some(GameState::Playing)));
}

pub fn start_new (
    mut next_state: ResMut<NextState<GameState>>,
) {
    next_state.set(GameState::MapGen);
}

pub fn hang (mut events: EventReader<AppExit>) {
    //if !events.is_empty() {
    //    events.clear();
        
        //while true is funnier than loop
        while true {
            println!("aaaaaaaa");
        }
    //    println!("Closing");
    //}
}


fn spawn_player(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    println!("spawnie?");

    commands
        // InputMap does not derive reflect, so we'll have to readd InputManagerBundle every time we reload.
        // This is okay, since we probably want our inputmap to be stored in an external 
        .spawn(InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new(INPUT_MAP),
        })
        .insert(Player)
        .insert(Position(IVec2::new(5, 5)))
        .insert(Renderable::new(Tile { glyph: '@', fg_color: Color::RED, bg_color: Color::NONE }, 64))
        .insert(Save);

    //next_state.set(GameState::MapGen);
    
}
 */