use std::fmt::Display;

pub struct MankallaGame {
    // 13 12 11 10  9  8  7
    //     0  1  2  3  4  5  6
    fields: [u8; 14],
    player_to_move: Player,
    finished: bool,
}

#[derive(PartialEq)]
pub enum Player {
    Player1,
    Player2,
}

impl Display for MankallaGame {
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

impl Default for MankallaGame {
    fn default() -> Self {
        MankallaGame {
            fields: [6, 6, 6, 6, 6, 6, 0, 6, 6, 6, 6, 6, 6, 0],
            player_to_move: Player::Player1,
            finished: false,
        }
    }
}

impl MankallaGame {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_points(&self, player: &Player) -> u8 {
        match player {
            Player::Player1 => self.fields[6],
            Player::Player2 => self.fields[13],
        }
    }

    fn handle_steal(&mut self, i: usize) {
        if self.fields[i] == 1 && self.player_to_move == Player::Player1 && i < 6 {
            self.fields[6] += self.fields[i] + self.fields[12 - i];
            self.fields[i] = 0;
            self.fields[12 - i] = 0;
        }
        if self.fields[i] == 1 && self.player_to_move == Player::Player2 && 6 < i && i < 13 {
            self.fields[13] += self.fields[i] + self.fields[12 - i];
            self.fields[i] = 0;
            self.fields[12 - i] = 0;
        }
    }

    fn handle_game_finished(&mut self) {
        let mut p1_sum = self.fields[0..6].iter().sum::<u8>();
        let mut p2_sum = self.fields[7..13].iter().sum::<u8>();

        if p1_sum != 0 && p2_sum != 0 {
            return;
        }

        p1_sum += self.fields[6];
        p2_sum += self.fields[13];

        for field in self.fields.iter_mut() {
            *field = 0;
        }
        self.fields[6] = p1_sum;
        self.fields[13] = p2_sum;

        self.finished = true;
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

    pub fn make_move(&mut self, slot: u8) {
        let mut i: usize = match self.player_to_move {
            Player::Player1 => {
                assert!(slot < 6);
                slot as usize
            }
            Player::Player2 => {
                assert!(slot < 6);
                (slot + 7) as usize
            }
        };

        let mut marbles_to_move = self.fields[i];
        self.fields[i] = 0;
        while marbles_to_move > 0 {
            i = (i + 1) % 14;
            self.fields[i] += 1;
            marbles_to_move -= 1;
        }

        self.handle_steal(i);

        self.handle_game_finished();

        self.handle_switch_player(i);
    }
}
