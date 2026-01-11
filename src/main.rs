use std::{
    error::Error,
    fs,
    io::{self, Stdin},
};

use mankalla_rl::{
    mankalla::{MankallaGame, MankallaGameState, Player},
    q_learning::{Deserialize, Environment, EpsilonGreedyPolicy, Policy, QLearning, Serialize},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut policy = match fs::read_to_string("policy.csv") {
        Ok(s) => EpsilonGreedyPolicy::<MankallaGame>::deserialize(s.as_str())?,
        Err(_) => EpsilonGreedyPolicy::<MankallaGame>::new(0.2, 1., 1., 0.1, -0.01),
    };

    // QLearning::train(&mut policy, 1000, None);

    game_loop(&mut policy);

    fs::write("policy.csv", policy.serialize())?;

    Ok(())
}

fn game_loop(policy: &mut impl Policy<MankallaGame>) {
    let mut turn: usize = 1;
    let mut state = MankallaGame::new();
    let mut finished = false;

    println!("{}", state);

    let stdin = io::stdin();

    let action;
    match get_player_input(&stdin) {
        PlayerRequest::Action(a) => {
            action = a;
        }
        PlayerRequest::Quit => {
            println!("Ok, goodbye");
            return;
        }
    };

    (state, finished) = player_turn(state, action, policy, &mut turn);
    while !finished {
        match state.get_player_to_move() {
            Player::Player2 => {
                (state, finished) = bot_turn(state, policy, &mut turn);
            }
            Player::Player1 => {
                let action;
                match get_player_input(&stdin) {
                    PlayerRequest::Action(a) => {
                        action = a;
                    }
                    PlayerRequest::Quit => {
                        println!("Ok, goodbye");
                        return;
                    }
                };

                (state, finished) = player_turn(state, action, policy, &mut turn);
            }
        }
    }
}

enum PlayerRequest {
    Action(u8),
    Quit,
}

fn get_player_input(stdin: &Stdin) -> PlayerRequest {
    println!("Choose your action: (0,1,2,3,4,5,q)");

    let mut input = String::new();
    loop {
        stdin
            .read_line(&mut input)
            .expect("Something with stdin went wrong");

        match input.as_str().strip_suffix("\n").unwrap_or("") {
            digit @ ("0" | "1" | "2" | "3" | "4" | "5") => {
                return PlayerRequest::Action(digit.parse().expect("Guaranteed to work"));
            }
            "q" => return PlayerRequest::Quit,
            _ => continue,
        }
    }
}

fn player_turn(
    state: MankallaGameState,
    action: u8,
    policy: &mut impl Policy<MankallaGame>,
    turn: &mut usize,
) -> (MankallaGameState, bool) {
    println!("Turn {turn}, you chose {action}");

    let (next_state, reward, finished) = MankallaGame::step(&state, &action);
    println!("{}", next_state);
    policy.improve(state.into(), action, reward, next_state, finished);

    *turn += 1;

    (next_state, finished)
}

fn bot_turn(
    state: MankallaGameState,
    policy: &mut impl Policy<MankallaGame>,
    turn: &mut usize,
) -> (MankallaGameState, bool) {
    let action = policy.choose_action(state.into());

    println!("Turn {turn}, bot chose {action}");

    let (next_state, reward, finished) = MankallaGame::step(&state, &action);
    println!("{}", next_state);
    policy.improve(state.into(), action, reward, next_state, finished);

    *turn += 1;

    (next_state, finished)
}
