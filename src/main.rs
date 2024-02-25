#![cfg_attr(
    all(
      target_os = "windows",
      not(feature = "console"),
    ),
    windows_subsystem = "windows"
)]

#![feature(const_fn_floating_point_arithmetic)]

use bevy::{prelude::*, ecs::schedule::ScheduleLabel, window::{exit_on_primary_closed, exit_on_all_closed}, app::AppExit};
use leafwing_input_manager::prelude::*;
use moonshine_save::{save::SavePlugin, load::LoadPlugin};
//use sark_grids::Grid;

#[path = "spatial/spatial.rs"]
mod spatial;
use spatial::*;

/*
#[path = "actions/actions.rs"]
mod actions;
use actions::*;

#[path = "log/log.rs"]
mod log;
use log::*;
 */

#[path = "map/map.rs"]
mod map;
use map::*;

/*
#[path = "player/player.rs"]
mod player;
use player::*;

#[path = "rendering/rendering.rs"]
mod rendering;
use rendering::*;

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

fn main () {
    App::new()
    .add_plugins(DefaultPlugins)
    // This plugin maps inputs to an input-type agnostic action-state
    // We need to provide it with an enum which stores the possible actions a player could take
    .add_plugins(InputManagerPlugin::<Action>::default())
    // The InputMap and ActionState components will be added to any entity with the Player component


    .init_resource::<RNGSeed>()

    .add_systems(Startup, map::generate_small_map)
    
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

#[derive(Clone, Copy, Resource, Deref, DerefMut, Reflect)]
pub struct RNGSeed(u32);
impl Default for RNGSeed {
    fn default() -> Self {
        Self(17)
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    MoveForward, MoveBackward,
    MoveLeft, MoveRight,
    Crouch, Jump,
}

const INPUT_MAP: [(KeyCode, Action); 6] = [(KeyCode::W, Action::MoveForward), (KeyCode::S, Action::MoveBackward),
                                            (KeyCode::A, Action::MoveLeft), (KeyCode::D, Action::MoveRight),
                                            (KeyCode::ShiftLeft, Action::Crouch), (KeyCode::Space, Action::Jump),
                                          ];

pub fn app_exit (mut events: EventReader<AppExit>) -> bool {
    !events.is_empty()
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