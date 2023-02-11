use std::time::Duration;

use bevy::prelude::*;

pub struct TurnSystemPlugin;

impl Plugin for TurnSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TurnOverEvent>()
            .add_startup_system_to_stage(StartupStage::PreStartup, setup_turn_sytem)
            .add_system_to_stage(CoreStage::PreUpdate, turn_system);
    }
}

// NOTE: Initial state of the `GameState` resource.
const INITIAL_GAME_STATE: GameStateID = GameStateID::Active;
// NOTE: Time between turns in milliseconds.
const TURN_INTERVAL: u64 = 400;

// NOTE: Event that is triggered at the end of every turn.
pub struct TurnOverEvent;

#[allow(dead_code)]
// NOTE: Every possible game state.
pub enum GameStateID {
    None,
    Active,
    Pause,
}

// NOTE: Resource that holds the current state of the game.
#[derive(Resource)]
pub struct GameState {
    pub state: GameStateID,
    pub timer: Timer,
}

// NOTE: Sets up the turn system and creates the `GameState` resource.
fn setup_turn_sytem(mut commands: Commands) {
    commands.insert_resource(GameState {
        state: INITIAL_GAME_STATE,
        timer: Timer::new(Duration::from_millis(TURN_INTERVAL), TimerMode::Repeating),
    });
}

// NOTE: Tracks the remaining time of the current turn, and
//       sends a `TurnOverEvent` if the timer is out.
fn turn_system(
    mut game_state: ResMut<GameState>,
    mut event_writer: EventWriter<TurnOverEvent>,
    time: Res<Time>,
) {
    if let GameStateID::Active = game_state.state {
        game_state.timer.tick(time.delta());

        if game_state.timer.just_finished() {
            event_writer.send(TurnOverEvent);
        }
    }
}
