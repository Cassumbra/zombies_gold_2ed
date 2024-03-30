#![cfg_attr(
    all(
      target_os = "windows",
      not(feature = "console"),
    ),
    windows_subsystem = "windows"
)]

#![feature(const_fn_floating_point_arithmetic)]

use bevy::{app::AppExit, ecs::schedule::ScheduleLabel, pbr::wireframe::WireframePlugin, prelude::*, render::texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor}};
use bevy_asset_loader::prelude::*;
use fastrand::Rng;
//use bevy_flycam::PlayerPlugin;
use leafwing_input_manager::prelude::*;
use moonshine_save::{save::SavePlugin, load::LoadPlugin};
//use bevy::render::settings::WgpuSettings;


#[path = "spatial/spatial.rs"]
mod spatial;
use spatial::*;


#[path = "actions/actions.rs"]
mod actions;
use actions::*;

#[path = "inventory/inventory.rs"]
mod inventory;
use inventory::*;

/*
#[path = "log/log.rs"]
mod log;
use log::*;
 */

#[path = "map/map.rs"]
mod map;
use map::*;

#[path = "physics/physics.rs"]
mod physics;
use physics::*;

#[path = "player/player.rs"]
mod player;
use player::*;

#[path = "rendering/rendering.rs"]
mod rendering;
use rendering::*;

#[path = "ui/ui.rs"]
mod ui;
use ui::*;

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

const CHUNK_SIZE: i32 = 16;
const WORLD_SIZE: [i32; 2] = [255; 2];
/// Surface height.
const WORLD_HEIGHT: i32 = 4;
/// Underground depth.
const WORLD_DEPTH: i32 = 12;

const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_WIDTH: f32 = 0.4;



fn main () {
    let title = if Rng::new().f32() > 0.90 {"3D Miner GOLD: Stones of Wealth and Perlin"} else {"3D Miner GOLD: Stones of Wealth and Peril"};
    let mut default_sampler =  ImageSamplerDescriptor::nearest();
    //default_sampler.mipmap_filter = ImageFilterMode::Linear;

    App::new()
    //.insert_resource(WgpuOptions {
    //    features: WgpuFeatures::POLYGON_MODE_LINE,
    //    ..Default::default()
    //})
    .add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title.into(),
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin { default_sampler })
    )
    .add_plugins(WireframePlugin)
    .insert_resource(Msaa::Off)

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
    .add_plugins(MapPlugin)

    .init_resource::<RNGSeed>()

    
    .add_systems(Startup, setup)
    .add_systems(Startup, setup_ui)

    .add_systems(Update, map::update_chunk_positions)
    .add_systems(Update, map::update_chunk_loaders)
    .add_systems(Update, map::generate_trees.before(generate_chunks))
    .add_systems(Update, map::generate_chunks)
    //.add_systems(Update, map::read_modification_events)
    .add_systems(Update, map::unload_chunks)

    .add_systems(Update, rendering::update_chunk_meshes.run_if(in_state(GameState::Playing)))
    .add_systems(Update, move_to_spawn.run_if(in_state(GameState::Playing)))
    .add_systems(Update, mining)
    .add_systems(Update, damage_block)
    .add_systems(Update, building)
    .add_systems(Update, place_block)
    
    .add_systems(Update, player_input_game)
    
    .add_systems(
        Update,
        (
            player_input_game,
            movement::movement,
            apply_friction,
            apply_gravity,
            do_physics,
        )
            .chain(),
    )

    .add_systems(Update, update_resource_counts.run_if(in_state(GameState::Playing)))
    .add_systems(Update, update_progress_bar)
     

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

/// Indicates that an entity needs to be moved to a safe position near the origin or to their set spawn.
#[derive(Clone, Copy, Component, Reflect)]
pub struct MoveToSpawn;

const SPAWN_CHUNK: IVec3 = IVec3::new(5, 0, 0);

pub fn move_to_spawn (
    mut query: Query<(Entity, &mut Transform), With<MoveToSpawn>>,

    chunk_map: Res<ChunkMap>,

    mut commands: Commands,

    mut evw_load_chunk: EventWriter<LoadChunkEvent>,
) {
    //println!("time to move arounda!");
    'entity_checks: for (entity, mut transform) in &mut query {
        'chunks: for chunk_y in (0..=WORLD_HEIGHT).rev() {
            //println!("chunk_y: {}", chunk_y);
            if let Some(chunk) = chunk_map.get(&IVec3::new(SPAWN_CHUNK.x, chunk_y, SPAWN_CHUNK.z)) {
                for (y, block) in chunk.blocks.iter_column(0, 0).enumerate().rev() {
                    if block.id != BlockID::Air {
                        transform.translation = IVec3::new(SPAWN_CHUNK.x * CHUNK_SIZE, chunk_y * CHUNK_SIZE + y as i32 + 2, SPAWN_CHUNK.z * CHUNK_SIZE).as_vec3();
                        //println!("translation: {}", transform.translation);
                        commands.entity(entity).remove::<MoveToSpawn>();
                        continue 'entity_checks
                    }
                }
                continue 'chunks
            }
            evw_load_chunk.send(LoadChunkEvent { chunk: IVec3::new(SPAWN_CHUNK.x, chunk_y, SPAWN_CHUNK.z), load_reason: LoadReason::Spawning(entity) });
            continue 'entity_checks
        }

        transform.translation = IVec3::new(SPAWN_CHUNK.x * CHUNK_SIZE, WORLD_HEIGHT * CHUNK_SIZE + 2, SPAWN_CHUNK.z * CHUNK_SIZE).as_vec3();
        commands.entity(entity).remove::<MoveToSpawn>();
        //todo!("We need to try some other spawn locations!");
    }
}

//let mut material = StandardMaterial::from(Color::WHITE);
//material.unlit = true;
//material.base_color_texture = Some(atlas.res_8x8.clone());

#[derive(AssetCollection, Resource)]
struct Atlas{
    #[asset(path = "textures_8x8.png")]
    pub res_8x8: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 8., tile_size_y = 8., columns = 32, rows = 32, padding_x = 0., padding_y = 0., offset_x = 0., offset_y = 0.))]
    pub items_8x8_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "items_8x8.png")]
    pub items_8x8: Handle<Image>,
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
    Look, Primary, Secondary,
    MenuBack,
}

const INPUT_MAP: [(Action, InputKind); 10] = [(Action::MoveForward, InputKind::PhysicalKey(KeyCode::KeyW)), (Action::MoveBackward, InputKind::PhysicalKey(KeyCode::KeyS)),
                                            (Action::MoveLeft, InputKind::PhysicalKey(KeyCode::KeyA)), (Action::MoveRight, InputKind::PhysicalKey(KeyCode::KeyD)),
                                            (Action::Crouch, InputKind::PhysicalKey(KeyCode::ShiftLeft)), (Action::Jump, InputKind::PhysicalKey(KeyCode::Space)),
                                            (Action::Look, InputKind::DualAxis(DualAxis::mouse_motion())), (Action::Primary, InputKind::Mouse(MouseButton::Left)), (Action::Secondary, InputKind::Mouse(MouseButton::Right)),
                                            (Action::MenuBack, InputKind::PhysicalKey(KeyCode::Escape)),
                                          ];

pub fn app_exit (mut events: EventReader<AppExit>) -> bool {
    !events.is_empty()
}



pub fn setup(
    mut commands: Commands,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    //assets: Res<AssetServer>,
) {

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
        transform: Transform::from_xyz(0.0, (PLAYER_HEIGHT * 0.9) / 2.0, 0.0),
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
        movement::CharacterControllerBundle::new(AabbCollider::new(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH)).with_movement(
            9.0,
            6.0,
        ),
        LinearVelocity::default(),
        Gravity(14.0),
        Player,
        MoveToSpawn,
        InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new(INPUT_MAP),
        },
        //Transform::default(),
        //GlobalTransform::default(),
        ChunkPosition::default(),
        ChunkLoader { range: 5, load_list: vec![] },
        MiningTimer::default(),
        BuildingTimer::default(),
        Inventory::default(),
    ))
    .add_child(camera);
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