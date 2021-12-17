use bevy::{ecs::schedule::ShouldRun, prelude::*};

// region [rgba(256,256,0,0.2)] State Functions

pub const IS_AIMING_LASER_STATE: for<'r> fn(bevy::prelude::Res<'r, GameStateRes>) -> ShouldRun =
    is_game_state::<{ GameState::AimingLaser }>;
pub const IS_MOVE_PLAYER_STATE: for<'r> fn(bevy::prelude::Res<'r, GameStateRes>) -> ShouldRun =
    is_game_state::<{ GameState::MovePlayer }>;
pub const IS_FIRE_LASER_STATE: for<'r> fn(bevy::prelude::Res<'r, GameStateRes>) -> ShouldRun =
    is_game_state::<{ GameState::FireLaser }>;
pub const IS_GENERATE_OBSTACLE_STATE: for<'r> fn(
    bevy::prelude::Res<'r, GameStateRes>,
) -> ShouldRun = is_game_state::<{ GameState::GenerateObstacle }>;
pub const IS_SPAWN_OBSTACLE_STATE: for<'r> fn(bevy::prelude::Res<'r, GameStateRes>) -> ShouldRun =
    is_game_state::<{ GameState::SpawnObstacle }>;
pub const IS_MOVE_OBSTACLE_STATE: for<'r> fn(bevy::prelude::Res<'r, GameStateRes>) -> ShouldRun =
    is_game_state::<{ GameState::MoveObstacle }>;

fn is_game_state<const T: GameState>(game_state: Res<GameStateRes>) -> ShouldRun {
    if game_state.state.eq(&T) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
//endregion

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameStateRes::default());
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum GameState {
    None,
    AimingLaser,
    MovePlayer,
    FireLaser,
    GenerateObstacle,
    SpawnObstacle,
    MoveObstacle,
}

pub struct GameStateRes {
    state: GameState,
}

impl GameStateRes {
    pub fn change(&mut self, game_state: GameState) {
        info!("Changed Game State ({:?})", game_state);
        self.state = game_state;
    }
}

impl Default for GameStateRes {
    fn default() -> Self {
        Self {
            state: GameState::GenerateObstacle,
        }
    }
}
