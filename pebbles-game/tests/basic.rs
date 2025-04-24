#![no_std]

use gtest::{Program, System};
use pebbles_game_io::*;

const USER: u64 = 102;

#[test]
fn test_init() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    sys.mint_to(USER, 10000000000000000);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };
    program.send(USER, pebbles_init);
    sys.run_next_block();

    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.pebbles_count, 15);
    assert_eq!(state.max_pebbles_per_turn, 2);
    assert!(state.pebbles_remaining <= 15 && state.pebbles_remaining >= 13); // Program may take 1-2 if first
    assert_eq!(state.difficulty, DifficultyLevel::Easy);
    assert!(state.winner.is_none());
}

#[test]
fn test_user_turn_easy() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    sys.mint_to(USER, 10000000000000000);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };
    program.send(USER, pebbles_init);
    sys.run_next_block();

    program.send(USER, PebblesAction::Turn(1));
    sys.run_next_block();

    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert!(state.pebbles_remaining <= 14 && state.pebbles_remaining >= 12); // Program removes 1 or 2
    assert!(state.winner.is_none());
}

#[test]
fn test_hard_mode_strategy() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    sys.mint_to(USER, 10000000000000000);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 15,
        max_pebbles_per_turn: 2,
    };
    program.send(USER, pebbles_init);
    sys.run_next_block();

    program.send(USER, PebblesAction::Turn(2));
    sys.run_next_block();

    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.pebbles_remaining, 12); // Program removes 3 to reach 10 (multiple of 3)
    assert!(state.winner.is_none());
}

#[test]
fn test_user_wins() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    sys.mint_to(USER, 10000000000000000);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 4,
        max_pebbles_per_turn: 2,
    };
    program.send(USER, pebbles_init);
    sys.run_next_block();

    program.send(USER, PebblesAction::Turn(1));
    sys.run_next_block();

    program.send(USER, PebblesAction::Turn(2));
    sys.run_next_block();
    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.pebbles_remaining, 0);
    assert_eq!(state.pebbles_count, 4);
    assert_eq!(state.max_pebbles_per_turn, 2);
    assert_eq!(state.winner, Some(Player::User));
}

#[test]
fn test_program_wins() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    sys.mint_to(USER, 10000000000000000);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 4,
        max_pebbles_per_turn: 3,
    };
    program.send(USER, pebbles_init);
    sys.run_next_block();

    program.send(USER, PebblesAction::Turn(1));
    sys.run_next_block();

    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.pebbles_remaining, 0);
    assert_eq!(state.winner, Some(Player::Program));
}

#[test]
fn test_give_up() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    sys.mint_to(USER, 10000000000000000);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 10,
        max_pebbles_per_turn: 3,
    };
    program.send(USER, pebbles_init);
    sys.run_next_block();

    program.send(USER, PebblesAction::GiveUp);
    sys.run_next_block();

    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.winner, Some(Player::Program));
}

#[test]
fn test_restart() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    sys.mint_to(USER, 10000000000000000);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 4,
        max_pebbles_per_turn: 3,
    };
    program.send(USER, pebbles_init);
    sys.run_next_block();

    program.send(USER, PebblesAction::Turn(1));
    sys.run_next_block();

    let state: GameState = program
        .read_state(())
        .expect("Failed to read appuntamenti state");
    assert_eq!(state.pebbles_remaining, 0);
    assert_eq!(state.winner, Some(Player::Program));

    program.send(
        USER,
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 6,
            max_pebbles_per_turn: 2,
        },
    );
    sys.run_next_block();

    let state: GameState = program.read_state(()).expect("Failed to read state");
    assert_eq!(state.pebbles_count, 6);
    assert_eq!(state.max_pebbles_per_turn, 2);
    assert_eq!(state.winner, None);
}
