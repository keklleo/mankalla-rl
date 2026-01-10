use crate::q_learning::{Deserialize, DeserializeError, Environment, Serialize};
use std::fmt::Display;

pub struct MankallaGame;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MankallaGameState {
    // 13 12 11 10  9  8  7
    //     0  1  2  3  4  5  6
    fields: [u8; 14],
    player_to_move: Player,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

impl Environment for MankallaGame {
    type State = MankallaGameState;
    type ActionRelevantState = [u8; 12];
    type Action = u8;

    fn new() -> MankallaGameState {
        Default::default()
    }

    fn actions(state: &Self::ActionRelevantState) -> Vec<Self::Action> {
        state[..6]
            .iter()
            .enumerate()
            .filter(|&(_, num_marbles)| *num_marbles > 0)
            .map(|(i, _)| i as u8)
            .collect()
    }

    fn step(state: &Self::State, action: &Self::Action) -> (Option<Self::State>, f32) {
        let mut state = state.clone();

        let p1_points = state.get_points(&Player::Player1);
        let p2_points = state.get_points(&Player::Player2);

        let mut i: usize = match state.player_to_move {
            Player::Player1 => {
                assert!(*action < 6);
                *action as usize
            }
            Player::Player2 => {
                assert!(*action < 6);
                (*action + 7) as usize
            }
        };

        let mut marbles_to_move = state.fields[i];
        state.fields[i] = 0;
        while marbles_to_move > 0 {
            i = (i + 1) % 14;
            state.fields[i] += 1;
            marbles_to_move -= 1;
        }

        state.handle_steal(i);

        let finished = state.handle_if_game_finished();

        let mut reward = (state.get_points(&Player::Player1) - p1_points) as f32
            - (state.get_points(&Player::Player2) - p2_points) as f32;
        if state.player_to_move == Player::Player2 {
            reward *= -1f32;
        }

        if finished {
            return (None, reward);
        }

        state.handle_switch_player(i);

        return (Some(state), reward);
    }
}

impl Serialize for [u8; 12] {
    fn serialize(&self) -> String {
        self.iter()
            .map(u8::to_string)
            .reduce(|a, b| format!("{} {}", a, b))
            .expect("I do not see how this slice could ever be empty")
    }
}

impl Deserialize for [u8; 12] {
    fn deserialize(input: &str) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let mut count = 0;
        let mut values: [u8; 12] = Default::default();
        let elems = input
            .split(' ')
            .inspect(|_| count += 1)
            .map(|a| match a.parse::<u8>() {
                Ok(v) => Ok(v),
                Err(_) => Err(DeserializeError),
            });

        for (i, elem) in elems.enumerate() {
            if i >= 12 {
                return Err(DeserializeError);
            }
            match elem {
                Ok(v) => values[i] = v,
                Err(e) => return Err(e),
            }
        }

        if count != 12 {
            return Err(DeserializeError);
        }

        Ok(values)
    }
}

impl Serialize for u8 {
    fn serialize(&self) -> String {
        self.to_string()
    }
}

impl Deserialize for u8 {
    fn deserialize(input: &str) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        match input.parse::<u8>() {
            Ok(v) => Ok(v),
            Err(_) => Err(DeserializeError),
        }
    }
}

impl From<MankallaGameState> for [u8; 12] {
    fn from(value: MankallaGameState) -> Self {
        let temp = match value.player_to_move {
            Player::Player1 => [&value.fields[..6], &value.fields[7..13]],
            Player::Player2 => [&value.fields[7..13], &value.fields[..6]],
        };
        temp.concat()
            .try_into()
            .expect("This should always be of length 12 by design")
    }
}

impl Display for MankallaGameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result: String = "".to_owned();
        result.push_str(
            self.fields[7..14]
                .iter()
                .rev()
                .map(|field| format!("{:>2}", field))
                .collect::<String>()
                .as_str(),
        );
        result.push_str("\n  ");
        result.push_str(
            self.fields[..7]
                .iter()
                .map(|field| format!("{:>2}", field))
                .collect::<String>()
                .as_str(),
        );
        write!(f, "{}", result)
    }
}

impl Default for MankallaGameState {
    fn default() -> Self {
        MankallaGameState {
            fields: [6, 6, 6, 6, 6, 6, 0, 6, 6, 6, 6, 6, 6, 0],
            player_to_move: Player::Player1,
        }
    }
}

impl MankallaGameState {
    fn get_points(&self, player: &Player) -> u8 {
        match player {
            Player::Player1 => self.fields[6],
            Player::Player2 => self.fields[13],
        }
    }

    fn handle_steal(&mut self, i: usize) {
        if self.fields[i] == 1
            && self.player_to_move == Player::Player1
            && i < 6
            && self.fields[12 - i] > 0
        {
            self.fields[6] += self.fields[i] + self.fields[12 - i];
            self.fields[i] = 0;
            self.fields[12 - i] = 0;
        }
        if self.fields[i] == 1
            && self.player_to_move == Player::Player2
            && 6 < i
            && i < 13
            && self.fields[12 - i] > 0
        {
            self.fields[13] += self.fields[i] + self.fields[12 - i];
            self.fields[i] = 0;
            self.fields[12 - i] = 0;
        }
    }

    fn handle_if_game_finished(&mut self) -> bool {
        let mut p1_sum = self.fields[0..6].iter().sum::<u8>();
        let mut p2_sum = self.fields[7..13].iter().sum::<u8>();

        if p1_sum != 0 && p2_sum != 0 {
            return false;
        }

        p1_sum += self.fields[6];
        p2_sum += self.fields[13];

        for field in self.fields.iter_mut() {
            *field = 0;
        }
        self.fields[6] = p1_sum;
        self.fields[13] = p2_sum;

        return true;
    }

    fn handle_switch_player(&mut self, i: usize) {
        if self.player_to_move == Player::Player1 && i != 6
            || self.player_to_move == Player::Player2 && i != 13
        {
            self.player_to_move = match self.player_to_move {
                Player::Player1 => Player::Player2,
                Player::Player2 => Player::Player1,
            }
        }
    }
}
