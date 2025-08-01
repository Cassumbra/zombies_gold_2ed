#![cfg_attr(
    all(
      target_os = "windows",
      not(feature = "console"),
    ),
    windows_subsystem = "windows"
)]

//#![feature(const_fn_floating_point_arithmetic)]

//use std::f32::consts::PI;

use std::{collections::BTreeMap, thread, time::Duration};

use bevy::{app::AppExit, ecs::schedule::ScheduleLabel, log::LogPlugin, pbr::wireframe::WireframePlugin, prelude::*, render::{camera::RenderTarget, render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages}, texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor}, Render, RenderSet}, utils::HashMap, window::WindowResolution};
use bevy::transform::TransformSystem::TransformPropagate;
use bevy_asset_loader::prelude::*;
//use bevy_mod_mipmap_generator::{generate_mipmaps, MipmapGeneratorPlugin, MipmapGeneratorSettings};
use fastrand::Rng;
use iyes_perf_ui::PerfUiPlugin;
//use bevy_flycam::PlayerPlugin;
use leafwing_input_manager::prelude::*;
//use moonshine_save::{save::SavePlugin, load::LoadPlugin};
//use image::{imageops::FilterType};
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

#[path = "mechanics/mechanics.rs"]
mod mechanics;
use mechanics::*;

#[path = "physics/physics.rs"]
mod physics;
use physics::*;

#[path = "player/player.rs"]
mod player;
use player::*;

#[path = "rendering/rendering.rs"]
mod rendering;
use rendering::*;

#[path = "stats/stats.rs"]
mod stats;
use stats::*;

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
    //default_sampler.anisotropy_clamp = 1;
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
            close_when_requested: false,
            ..default()
        }).set(LogPlugin {
            filter: "wgpu=error,naga=warn".into(), //,bevy_ecs=debug
            ..default()
        })
        .set(ImagePlugin { default_sampler })
        /*
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(256.0, 192.0).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        })
         */
    )
    /*
    .insert_resource(MipmapGeneratorSettings {
        anisotropic_filtering: 1,
        filter_type: FilterType::Nearest,
        ..default()
    })
    .add_plugins(MipmapGeneratorPlugin)
    .add_systems(Update, generate_mipmaps::<StandardMaterial>)
      */
    .add_plugins(WireframePlugin)
    .insert_resource(Msaa::Off)
    .insert_resource(ClearColor(Color::Rgba { red: 129.0/256.0, green: 194.0/256.0, blue: 247.0/256.0, alpha: 1.0 }))

    .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Playing)
                .load_collection::<Atlas>()
                .load_collection::<Materials>(),
        )

    .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
    .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
    .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
    
    .add_plugins(PerfUiPlugin)
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

    .add_plugins(PhysicsPlugin)
    .add_plugins(ActionsPlugin)
    .add_plugins(MapPlugin)
    .add_plugins(StatsPlugin)
    .add_plugins(MechanicsPlugin)

    .init_resource::<RNGSeed>()

    
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GameState::Playing), setup_ui)

    .add_systems(OnEnter(GameState::Playing), modify_materials)
    .add_systems(PostUpdate, update_water_material.run_if(in_state(GameState::Playing)).after(TransformPropagate).before(RenderSet::PrepareAssets))

    .add_systems(Update, map::update_chunk_positions)
    .add_systems(Update, map::update_chunk_loaders)
    .add_systems(Update, map::generate_trees.before(generate_chunks))
    .add_systems(Update, map::generate_chunks)
    //.add_systems(Update, map::read_modification_events)
    // TODO: Chained just for exit/save reasons. We should add a state for exiting and saving (and also a state for pausing!)
    .add_systems(
        Update,
        (
            map::unload_chunks,
            map::save_chunks,
        )
            .chain(),
    )

    .add_systems(Update, rendering::update_chunk_meshes.run_if(in_state(GameState::Playing)))
    .add_systems(Update, move_to_spawn.run_if(in_state(GameState::Playing)))
    .add_systems(Update, mining)
    .add_systems(Update, damage_block)
    .add_systems(Update, building)
    .add_systems(Update, place_block)
    .add_systems(Update, process_block_updates)
    .add_systems(Update, mechanics::handle_breath)
    .add_systems(Update, mechanics::handle_suffocation)
    .add_systems(Update, mechanics::handle_fall_damage)
    .add_systems(Update, mechanics::handle_death)
    .add_systems(Update, stats::do_stat_change)

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
    .add_systems(Update, update_breath_ui.run_if(in_state(GameState::Playing)))
    .add_systems(Update, update_progress_bar)
    .add_systems(Update, fit_canvas)
     

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
    //mut chunk_status_map: ResMut<ChunkStatusMap>,
) {
    //println!("time to move arounda!");
    'entity_checks: for (entity, mut transform) in &mut query {
        'chunks: for chunk_y in (0..=WORLD_HEIGHT).rev() {
            //println!("chunk_y: {}", chunk_y);
            let chunk_pos = IVec3::new(SPAWN_CHUNK.x, chunk_y, SPAWN_CHUNK.z);
            if let Some(chunk) = chunk_map.get(&chunk_pos) {
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
            evw_load_chunk.send(LoadChunkEvent { chunk: chunk_pos, load_reason: LoadReason::Spawning(entity) });
            //chunk_status_map.insert(chunk_pos, ChunkStatus::Loading);
            continue 'entity_checks
        }

        transform.translation = IVec3::new(SPAWN_CHUNK.x * CHUNK_SIZE, WORLD_HEIGHT * CHUNK_SIZE + 2, SPAWN_CHUNK.z * CHUNK_SIZE).as_vec3();
        commands.entity(entity).remove::<MoveToSpawn>();
        //todo!("We need to try some other spawn locations!");
    }
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

pub fn setup (
    mut commands: Commands,

    mut images: ResMut<Assets<Image>>,
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

    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // Camera
    let camera_entity = commands.spawn(Camera3dBundle {
        camera: Camera {
            order: -1,
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        transform: Transform::from_xyz(0.0, (PLAYER_HEIGHT * 0.9) / 2.0, 0.0),
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),

            ..default()
        }),

        ..default()
    })
    .insert(InGameCamera)
    .id();

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((Camera2dBundle::default(), OuterCamera, HIGH_RES_LAYERS));

    // spawn the canvas
    commands.spawn((
        SpriteBundle {
            texture: image_handle,
            ..default()
        },
        Canvas,
        HIGH_RES_LAYERS,
    ));

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
        DistanceBeforeCollision::default(),
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
        ChunkLoader { range: 15, load_list: vec![] },
        MiningTimer::default(),
        BuildingTimer::default(),
        Inventory::default(),
        Stats(HashMap::from([
            (StatType::Health, Stat::new(0.0, 20.0)),
            (StatType::Breath, Stat::new(0.0, 100.0)),
        ])),
        HasAir(true),
    ))
    .add_child(camera_entity);
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