use std::{os::macos::raw::stat, time};

use rand::{distributions::Alphanumeric, random, Rng};

struct TimeKeeper {
    start_time_: std::time::Instant,
    time_threshold_: u64,
}
impl TimeKeeper {
    pub fn new(time_threshold: u64) -> Self {
        Self {
            start_time_: time::Instant::now(),
            time_threshold_: time_threshold,
        }
    }
    pub fn isTimeOver(&self) -> bool {
        self.start_time_.elapsed() > time::Duration::from_millis(self.time_threshold_)
    }
}

const H: i32 = 3;
const W: i32 = 3;
const END_TURN: usize = 4;

type ScoreType = i32;
const INF: i32 = 100000000;

pub enum WinningStatus {
    WIN,
    LOSE,
    DRAW,
    NONE,
}
#[derive(Debug, Default, Clone, Eq, PartialEq, Copy)]
struct Character {
    y_: i32,
    x_: i32,
    game_score_: i32,
}
impl Character {
    pub fn new(y_: i32, x_: i32) -> Self {
        Self {
            y_: y_,
            x_: x_,
            game_score_: 0,
        }
    }
}
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AlternateMazeState {
    points_: [[i32; W as usize]; H as usize],
    turn_: usize,
    characters_: Vec<Character>,
}

impl AlternateMazeState {
    const dx: [i32; 4] = [1, -1, 0, 0];
    const dy: [i32; 4] = [0, 0, 1, -1];
    pub fn new(seed: usize) -> Self {
        let mut state = Self {
            points_: [[0; W as usize]; H as usize],
            turn_: 0,
            characters_: vec![
                Character::new(H / 2, W / 2 - 1),
                Character::new(H / 2, W / 2 + 1),
            ],
        };
        let mut rng = rand::thread_rng();
        for y in 0..H {
            for x in 0..W {
                let point = rng.gen_range(0..10);
                if state.characters_[0].y_ == y && state.characters_[0].x_ == x {
                    continue;
                }
                if state.characters_[1].y_ == y && state.characters_[1].x_ == x {
                    continue;
                }
                state.points_[y as usize][x as usize] = point;
            }
        }
        state
    }
    fn isFirstPlayer(&self) -> bool {
        self.turn_ % 2 == 0
    }

    pub fn isDone(&self) -> bool {
        self.turn_ == END_TURN
    }
    pub fn advance(&mut self, action: usize) {
        let character = &mut self.characters_[0];
        character.x_ += Self::dx[action];
        character.y_ += Self::dy[action];
        let point: &mut i32 = &mut self.points_[character.y_ as usize][character.x_ as usize];
        if *point > 0 {
            character.game_score_ += *point;
            *point = 0;
        }
        self.turn_ += 1;
        self.characters_.swap(0, 1);
    }
    pub fn legalActions(&self) -> Vec<usize> {
        let mut actions = Vec::new();
        let character = &self.characters_[0];
        for action in 0..4 {
            let ty = character.y_ + Self::dy[action];
            let tx = character.x_ + Self::dx[action];
            if ty >= 0 && ty < H && tx >= 0 && tx < W {
                actions.push(action);
            }
        }
        actions
    }
    pub fn getWinningStatus(&self) -> WinningStatus {
        if self.isDone() {
            if self.characters_[0].game_score_ > self.characters_[1].game_score_ {
                return WinningStatus::WIN;
            } else if self.characters_[0].game_score_ < self.characters_[1].game_score_ {
                return WinningStatus::LOSE;
            } else {
                return WinningStatus::DRAW;
            }
        }
        WinningStatus::NONE
    }
    pub fn getScore(&self) -> ScoreType {
        return self.characters_[0].game_score_ - self.characters_[1].game_score_;
    }
    pub fn getFirstPlayerScoreForWinRate(&self) -> f64 {
        match self.getWinningStatus() {
            WinningStatus::WIN => {
                if self.isFirstPlayer() {
                    return 1.0;
                } else {
                    return 0.0;
                }
            }
            WinningStatus::LOSE => {
                if self.isFirstPlayer() {
                    return 0.0;
                } else {
                    return 1.0;
                }
            }
            _ => return 0.5,
        }
    }

    pub fn toString(&self) -> String {
        let mut ss = String::new();
        ss += format!("turn:\t{}\n", self.turn_).as_str();
        for player_id in 0..self.characters_.len() {
            let mut actual_player_id = player_id;
            if self.turn_ % 2 == 1 {
                actual_player_id = (player_id + 1) % 2;
            }
            let chara = &self.characters_[actual_player_id as usize];
            ss += format!(
                "score({})\t {}\ty:{} x:{}\n",
                player_id, chara.game_score_, chara.y_, chara.x_
            )
            .as_str();
        }
        for h in 0..H {
            for w in 0..W {
                let mut is_written = false;
                for player_id in 0..self.characters_.len() {
                    let mut actual_player_id = player_id;
                    if self.turn_ % 2 == 1 {
                        actual_player_id = (player_id + 1) % 2;
                    }
                    let character = &self.characters_[player_id as usize];
                    if character.y_ == h && character.x_ == w {
                        if actual_player_id == 0 {
                            ss += "A";
                        } else {
                            ss += "B";
                        }
                        is_written = true;
                    }
                }
                if !is_written {
                    if self.points_[h as usize][w as usize] > 0 {
                        ss += format!("{}", self.points_[h as usize][w as usize]).as_str();
                    } else {
                        ss += ".";
                    }
                }
            }
            ss += "\n";
        }

        ss
    }
}

type State = AlternateMazeState;
fn randomAction(state: &State) -> usize {
    let mut rng = rand::thread_rng();
    let legal_actions = state.legalActions();
    let id = rng.gen_range(0..legal_actions.len());
    return legal_actions[id as usize];
}

type AIFunction = fn(&State) -> usize;
type StringAIPair = (String, AIFunction);

fn testFirstPlayerWinRate(ais: [StringAIPair; 2], game_number: usize) {
    let mut first_player_win_rate = 0.0;
    for i in 0..game_number {
        let mut best_state = State::new(i);
        for j in 0..2 {
            let mut state = best_state.clone();
            let first_ai = &ais[j];
            let second_ai = &ais[(j + 1) % 2];
            loop {
                state.advance(first_ai.1(&state));
                if state.isDone() {
                    break;
                }
                state.advance(second_ai.1(&state));
                if state.isDone() {
                    break;
                }
            }
            let mut win_rate_point = state.getFirstPlayerScoreForWinRate();
            if j == 1 {
                win_rate_point = 1.0 - win_rate_point;
            }
            if win_rate_point >= 0.0 {
                state.toString();
            }
            first_player_win_rate += win_rate_point;
        }
        println!(
            "i {} w {} ",
            i,
            first_player_win_rate / ((i + 1) * 2) as f64
        );
    }
    first_player_win_rate /= (game_number * 2) as f64;
    println!(
        "Winning rate of {} to {} :\t {}",
        ais[0].0, ais[1].0, first_player_win_rate
    );
}
pub mod montecalro {
    use crate::randomAction;
    use crate::State;
    use crate::WinningStatus;
    use crate::INF;
    fn playout(state: &mut State) -> f64 {
        match state.getWinningStatus() {
            WinningStatus::WIN => return 1.0,
            WinningStatus::LOSE => return 0.0,
            WinningStatus::DRAW => return 0.5,
            _ => {
                state.advance(randomAction(state));
                return 1.0 - playout(state);
            }
        }
    }
    pub fn primitiveMontecarloAction(state: &State, playout_number: usize) -> usize {
        let legal_actions = state.legalActions();
        let mut values = vec![0.0; legal_actions.len()];
        let mut cnts = vec![0.0; legal_actions.len()];
        for cnt in 0..playout_number {
            let index = cnt % legal_actions.len();
            let mut next_state = state.clone();
            next_state.advance(legal_actions[index]);
            values[index] += 1.0 - playout(&mut next_state);
            cnts[index] += 1.0;
        }
        let mut best_action_index = 0;
        let mut best_score = -INF as f64;
        for index in 0..legal_actions.len() {
            let value_mean = values[index] / cnts[index];
            if value_mean > best_score {
                best_score = value_mean;
                best_action_index = index;
            }
        }
        legal_actions[best_action_index]
    }
}

fn main() {
    let f0: AIFunction = |state: &State| return montecalro::primitiveMontecarloAction(state, 3000);
    let f1: AIFunction = |state: &State| return randomAction(state);
    let ais = [
        ("primitiveMontecarloAction 3000".to_string(), f0),
        ("randomAction".to_string(), f1),
    ];
    testFirstPlayerWinRate(ais, 100);
}
