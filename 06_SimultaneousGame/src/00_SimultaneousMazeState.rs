use rand::{distributions::Standard, Rng};

const H: i32 = 3;
const W: i32 = 3;
const END_TURN: usize = 4;
const dstr: [&str; 4] = ["RIGHT", "LEFT", "DOWN", "UP"];

type ScoreType = i32;
const INF: i32 = 100000000;

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
pub struct SimultaneousMazeState {
    points_: [[i32; W as usize]; H as usize],
    turn_: usize,
    characters_: Vec<Character>,
}

impl SimultaneousMazeState {
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
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        for y in 0..H {
            for x in 0..W {
                let point = rng.gen_range(0..10);
                if state.characters_[0].y_ == y && state.characters_[0].x_ == x {
                    continue;
                }
                if state.characters_[1].y_ == y && state.characters_[1].x_ == x {
                    continue;
                }
                let mut ty = y;
                let mut tx = x;
                state.points_[ty as usize][tx as usize] = point;
                tx = W - 1 - x;
                state.points_[ty as usize][tx as usize] = point;
            }
        }
        state
    }

    pub fn isDone(&self) -> bool {
        self.turn_ == END_TURN
    }
    pub fn advance(&mut self, action0: usize, action1: usize) {
        {
            let character = &mut self.characters_[0];
            let action = action0;
            character.x_ += Self::dx[action];
            character.y_ += Self::dy[action];
            let point: &mut i32 = &mut self.points_[character.y_ as usize][character.x_ as usize];
            if *point > 0 {
                character.game_score_ += *point;
            }
        }
        {
            let character = &mut self.characters_[1];
            let action = action1;
            character.x_ += Self::dx[action];
            character.y_ += Self::dy[action];
            let point: &mut i32 = &mut self.points_[character.y_ as usize][character.x_ as usize];
            if *point > 0 {
                character.game_score_ += *point;
            }
        }
        for character_id in 0..self.characters_.len() {
            let character = &self.characters_[character_id];
            self.points_[character.y_ as usize][character.x_ as usize] = 0;
        }

        self.turn_ += 1;
    }
    pub fn legalActions(&self, player_id: usize) -> Vec<usize> {
        let mut actions = Vec::new();
        let character = &self.characters_[player_id];
        for action in 0..4 {
            let ty = character.y_ + Self::dy[action];
            let tx = character.x_ + Self::dx[action];
            if ty >= 0 && ty < H && tx >= 0 && tx < W {
                actions.push(action);
            }
        }
        actions
    }

    pub fn getScore(&self) -> ScoreType {
        return self.characters_[0].game_score_ - self.characters_[1].game_score_;
    }
    pub fn getScoreRate(&self) -> f64 {
        if self.characters_[0].game_score_ + self.characters_[1].game_score_ == 0 {
            return 0.0;
        }
        return self.characters_[0].game_score_ as f64
            / (self.characters_[0].game_score_ + self.characters_[1].game_score_) as f64;
    }

    pub fn toString(&self) -> String {
        let mut ss = String::new();
        ss += format!("turn:\t{}\n", self.turn_).as_str();
        for player_id in 0..self.characters_.len() {
            let chara = &self.characters_[player_id];
            ss += format!("score({})\t {}\n", player_id, chara.game_score_).as_str();
        }
        for h in 0..H {
            for w in 0..W {
                let mut is_written = false;
                for player_id in 0..self.characters_.len() {
                    let character = &self.characters_[player_id as usize];
                    if character.y_ == h && character.x_ == w {
                        if player_id == 0 {
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

type State = SimultaneousMazeState;
fn randomAction(state: &State, player_id: usize) -> usize {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0);
    let legal_actions = state.legalActions(player_id);
    let id = rng.gen_range(0..legal_actions.len());
    return legal_actions[id as usize];
}

type AIFunction = fn(&State, usize) -> usize;
type StringAIPair = (String, AIFunction);

fn playGame(ais: [StringAIPair; 2], seed: usize) {
    let mut state = State::new(seed);
    println!("{}", state.toString());
    while !state.isDone() {
        let actions = vec![ais[0].1(&state, 0), ais[1].1(&state, 1)];
        println!("actions {} {}", dstr[actions[0]], dstr[actions[1]]);
        state.advance(actions[0], actions[1]);
        println!("{}", state.toString());
    }
}

fn main() {
    let f0: AIFunction = |state: &State, seed: usize| return randomAction(state, seed);
    let f1: AIFunction = |state: &State, seed: usize| return randomAction(state, seed);
    let ais = [
        ("randomAction".to_string(), f0),
        ("randomAction".to_string(), f1),
    ];
    playGame(ais, 0);
}
