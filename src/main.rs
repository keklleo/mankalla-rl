use std::{
    error::Error,
    fs,
    io::{self, Stdin},
};

use mankalla_rl::{
    mankalla::MankallaGame,
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
    let state = MankallaGame::new();

    println!("{}", state);

    println!("Choose your action: (0,1,2,3,4,5,q)");

    let stdin = io::stdin();
    let action;
    loop {
        match get_player_input(&stdin) {
            PlayerRequest::Action(a) => {
                action = a;
                break;
            }
            PlayerRequest::Quit => {
                println!("Ok, goodbye");
                return;
            }
            PlayerRequest::Noop => {
                println!("Well you gotta do something");
                continue;
            }
            PlayerRequest::InvalidRequest => {
                println!("That is not something you can do");
                continue;
            }
        };
    }
    println!("Turn {turn}, you chose {action}");

    let (next_state, reward, finished) = MankallaGame::step(&state, &action);
    while !finished {
        // TODO: Bot turn

        turn += 1;

        // TODO: Player turn
    }
}

enum PlayerRequest {
    Action(u8),
    Quit,
    Noop,
    InvalidRequest,
}

fn get_player_input(stdin: &Stdin) -> PlayerRequest {
    let mut input = String::new();
    stdin
        .read_line(&mut input)
        .expect("Something with stdin went wrong");

    match input.as_str() {
        digit @ ("0" | "1" | "2" | "3" | "4" | "5") => {
            PlayerRequest::Action(digit.parse().expect("Guaranteed to work"))
        }
        "q" => PlayerRequest::Quit,
        "" => PlayerRequest::Noop,
        _ => PlayerRequest::InvalidRequest,
    }
}
