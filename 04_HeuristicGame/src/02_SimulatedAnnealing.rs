use core::num;
use std::arch::x86_64;

use rand;
use rand::distributions::Standard;
use rand::prelude::*;
const H: i32 = 5;
const W: i32 = 5;
const END_TURN: usize = 5;
const CHARACTER_N: usize = 3;
type ScoreType = i32;
const INF: ScoreType = 100000000;

#[derive(Debug, Default, Clone, Eq, PartialEq, Copy)]
struct Coord {
    y_: i32,
    x_: i32,
}
impl Coord {
    pub fn new(y_: i32, x_: i32) -> Self {
        Self { y_, x_ }
    }
}
#[derive(Debug, Default, Clone, Eq, PartialEq, Copy)]
struct AutoMoveMazeState {
    points_: [[i32; W as usize]; H as usize],
    turn_: i32,
    pub characters_: [Coord; CHARACTER_N],
    pub game_score_: i32,
    pub evaluated_score_: ScoreType,
}
impl AutoMoveMazeState {
    const dx: [i32; 4] = [1, -1, 0, 0];
    const dy: [i32; 4] = [0, 0, 1, -1];

    fn movePlayer(&mut self, character_id: usize) {
        let character = &mut self.characters_[character_id];
        let mut best_point = -INF;
        let mut best_action_index = 0;
        for action in 0..4 {
            let ty = character.y_ + Self::dy[action];
            let tx: i32 = character.x_ + Self::dx[action];
            if ty >= 0 && ty < H && tx >= 0 && tx < W {
                let point = self.points_[ty as usize][tx as usize];
                if point > best_point {
                    best_point = point;
                    best_action_index = action;
                }
            }
        }
        character.y_ += Self::dy[best_action_index];
        character.x_ += Self::dx[best_action_index];
    }
    fn advance(&mut self) {
        for character_id in 0..CHARACTER_N {
            self.movePlayer(character_id);
        }
        for character in self.characters_.iter() {
            let point = &mut self.points_[character.y_ as usize][character.x_ as usize];
            self.game_score_ += *point;
            *point = 0;
        }
        self.turn_ += 1;
    }

    pub fn new(seed: usize) -> Self {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        let mut maze_state = AutoMoveMazeState {
            points_: [[0 as i32; W as usize]; H as usize],
            turn_: 0,
            game_score_: 0,
            evaluated_score_: 0,
            characters_: [Coord::new(0, 0); CHARACTER_N],
        };
        for y in 0..H {
            for x in 0..W {
                maze_state.points_[y as usize][x as usize] = rng.gen_range(0..9) + 1;
            }
        }
        maze_state
    }

    pub fn setCharacter(&mut self, character_id: usize, y: i32, x: i32) {
        self.characters_[character_id].y_ = y;
        self.characters_[character_id].x_ = x;
    }

    pub fn isDone(&self) -> bool {
        return self.turn_ == END_TURN as i32;
    }
    pub fn toString(&self) -> String {
        let mut ss = String::new();
        ss += format!("turn:\t{}\n", self.turn_).as_str();
        ss += format!("score:\t{}\n", self.game_score_).as_str();
        let mut board_chars = [['.'; W as usize]; H as usize];
        for h in 0..H {
            for w in 0..W {
                let mut is_written = false;
                for character in self.characters_ {
                    if character.y_ == h && character.x_ == w {
                        ss += "@";
                        is_written = true;
                        break;
                    }
                    board_chars[character.y_ as usize][character.x_ as usize] = '@';
                }
                if !is_written {
                    if self.points_[h as usize][w as usize] > 0 {
                        ss += self.points_[h as usize][w as usize].to_string().as_str();
                    } else {
                        ss += ".";
                    }
                }
            }
            ss += "\n";
        }
        ss
    }

    pub fn getScore(&self, is_print: bool) -> ScoreType {
        let mut tmp_state = self.clone();
        for character in self.characters_ {
            let point = &mut tmp_state.points_[character.y_ as usize][character.x_ as usize];
            *point = 0;
        }
        while !tmp_state.isDone() {
            tmp_state.advance();
            if is_print {
                println!("{}", tmp_state.toString());
            }
        }
        tmp_state.game_score_
    }

    pub fn init(&mut self) {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0 as u64);

        for character in self.characters_.iter_mut() {
            character.y_ = rng.gen_range(0..H);
            character.x_ = rng.gen_range(0..W);
        }
    }
    pub fn transition(&mut self) {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0 as u64);
        let character = &mut self.characters_[rng.gen::<usize>() % CHARACTER_N];
        character.y_ = rng.gen_range(0..H);
        character.x_ = rng.gen_range(0..W);
    }
}

type State = AutoMoveMazeState;
fn randomAction(state: &State) -> State {
    let mut rng = rand::thread_rng();
    let mut now_state = state.clone();
    for character_id in 0..CHARACTER_N {
        let y = rng.gen_range(0..H);
        let x = rng.gen_range(0..W);
        now_state.setCharacter(character_id, y, x);
    }
    now_state
}
fn hillClimb(state: &State, number: i32) -> State {
    let mut now_state = state.clone();
    now_state.init();
    let mut best_score = now_state.getScore(false);
    for i in 0..number {
        let mut next_state = now_state.clone();
        now_state.transition();
        let next_score = next_state.getScore(false);
        if next_score > best_score {
            best_score = next_score;
            now_state = next_state;
        }
    }
    now_state
}

fn simulatedAnnealing(state: &State, number: i32, start_temp: f64, end_temp: f64) -> State {
    let mut rng = rand::thread_rng();
    let mut now_state = state.clone();
    now_state.init();
    let mut best_score = now_state.getScore(false);
    let mut now_score = best_score;
    let mut best_state = now_state.clone();
    for i in 0..number {
        let mut next_state = now_state.clone();
        now_state.transition();
        let next_score = next_state.getScore(false);
        let temp = start_temp + (end_temp - start_temp) * (i as f64 / number as f64);
        let probability = ((next_score - now_score) as f64 / temp).exp();
        let is_force_next = probability > (rng.gen_range(0..INF) as f64 / INF as f64);
        if next_score > now_score || is_force_next {
            now_score = next_score;
            now_state = next_state;
        }
        if next_score > best_score {
            best_score = next_score;
            now_state = next_state;
        }
    }
    now_state
}

// type AIFunction =
fn playGame(seed: usize) {
    let mut state = State::new(seed);
    state = hillClimb(&state, 10000);
    let score = state.getScore(true);
    println!("Score of hillClimb: {}", score);
}
fn testAiScore(game_number: usize) {
    let mut rng = rand::thread_rng();
    let mut score_mean = 0.0;

    for i in 0..game_number {
        let mut state = State::new(rng.gen_range(0..INF) as usize);
        state = simulatedAnnealing(&state, 10000, 500.0, 10.0);
        score_mean += state.getScore(false) as f64;
    }
    score_mean /= game_number as f64;
    println!("Score of Annewaling:\t {}", score_mean);
}

fn main() {
    testAiScore(10);
}
