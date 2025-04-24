#![no_std]
#![allow(static_mut_refs)]

use gstd::{exec, msg, prelude::*};
use pebbles_game_io::*;

static mut GAME_STATE: Option<GameState> = None;

#[cfg(not(test))]
fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

#[cfg(test)]
fn get_random_u32() -> u32 {
    42
}

#[no_mangle]
pub extern "C" fn init() {
    let init_msg: PebblesInit = msg::load().expect("Failed to decode PebblesInit");

    if init_msg.pebbles_count == 0 {
        panic!("Pebbles count must be greater than 0");
    }
    if init_msg.max_pebbles_per_turn == 0 {
        panic!("Max pebbles per turn must be greater than 0");
    }
    if init_msg.max_pebbles_per_turn > init_msg.pebbles_count {
        panic!("Max pebbles per turn cannot exceed total pebbles");
    }

    let first_player = if get_random_u32() % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };

    let mut game_state = GameState {
        pebbles_count: init_msg.pebbles_count,
        max_pebbles_per_turn: init_msg.max_pebbles_per_turn,
        pebbles_remaining: init_msg.pebbles_count,
        difficulty: init_msg.difficulty,
        first_player: first_player.clone(),
        winner: None,
    };

    if first_player == Player::Program {
        let pebbles_to_remove = program_turn(&game_state);
        game_state.pebbles_remaining -= pebbles_to_remove;

        if game_state.pebbles_remaining == 0 {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to send event");
        } else {
            msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0)
                .expect("Failed to send event");
        }
    }

    unsafe {
        GAME_STATE = Some(game_state);
    }
}

#[no_mangle]
pub extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Failed to decode PebblesAction");
    let mut game_state = unsafe { GAME_STATE.clone().expect("Game state not initialized") };

    //     if game_state.winner.is_some() {
    //         panic!("Game is already over");
    //     }

    match action {
        PebblesAction::Turn(count) => {
            if count == 0 {
                panic!("Cannot remove 0 pebbles");
            }
            if count > game_state.max_pebbles_per_turn {
                panic!("Cannot remove more than max pebbles per turn");
            }
            if count > game_state.pebbles_remaining {
                panic!("Cannot remove more pebbles than remaining");
            }

            game_state.pebbles_remaining -= count;

            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::User);
                msg::reply(PebblesEvent::Won(Player::User), 0).expect("Failed to send event");
            } else {
                let pebbles_to_remove = program_turn(&game_state);
                game_state.pebbles_remaining -= pebbles_to_remove;

                if game_state.pebbles_remaining == 0 {
                    game_state.winner = Some(Player::Program);
                    msg::reply(PebblesEvent::Won(Player::Program), 0)
                        .expect("Failed to send event");
                } else {
                    msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0)
                        .expect("Failed to send event");
                }
            }
        }
        PebblesAction::GiveUp => {
            if game_state.winner.is_some() {
                panic!("Game is already over");
            }
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to send event");
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            if pebbles_count == 0 {
                panic!("Pebbles count must be greater than 0");
            }
            if max_pebbles_per_turn == 0 {
                panic!("Max pebbles per turn must be greater than 0");
            }
            if max_pebbles_per_turn > pebbles_count {
                panic!("Max pebbles per turn cannot exceed total pebbles");
            }

            let first_player = if get_random_u32() % 2 == 0 {
                Player::User
            } else {
                Player::Program
            };

            game_state = GameState {
                pebbles_count,
                max_pebbles_per_turn,
                pebbles_remaining: pebbles_count,
                difficulty,
                first_player: first_player.clone(),
                winner: None,
            };

            if first_player == Player::Program {
                let pebbles_to_remove = program_turn(&game_state);
                game_state.pebbles_remaining -= pebbles_to_remove;

                if game_state.pebbles_remaining == 0 {
                    game_state.winner = Some(Player::Program);
                    msg::reply(PebblesEvent::Won(Player::Program), 0)
                        .expect("Failed to send event");
                } else {
                    msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0)
                        .expect("Failed to send event");
                }
            }
        }
    }

    unsafe {
        GAME_STATE = Some(game_state);
    }
}

#[no_mangle]
pub extern "C" fn state() {
    let game_state = unsafe { GAME_STATE.clone().expect("Game state not initialized") };
    msg::reply(game_state, 0).expect("Failed to reply with game state");
}

fn program_turn(game_state: &GameState) -> u32 {
    match game_state.difficulty {
        DifficultyLevel::Easy => {
            let max = game_state
                .max_pebbles_per_turn
                .min(game_state.pebbles_remaining);
            (get_random_u32() % max) + 1
        }
        DifficultyLevel::Hard => {
            let target = game_state.pebbles_remaining % (game_state.max_pebbles_per_turn + 1);
            if target == 0 {
                1
            } else {
                target
            }
        }
    }
}
