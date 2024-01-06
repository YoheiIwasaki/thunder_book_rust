use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::time;

use rand;
use rand::prelude::*;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct Coord {
    y_: i32,
    x_: i32,
}
impl Coord {
    pub fn new(y_: i32, x_: i32) -> Self {
        Self { y_, x_ }
    }
}

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

type ScoreType = i32;
const INF: ScoreType = 10000000;

const H: i32 = 7;
const W: i32 = 7;
const END_TURN: i32 = 49;
mod zobrist_hash {
    use crate::H;
    use crate::W;
    use lazy_static::lazy_static;
    use rand;
    use rand::prelude::*;

    lazy_static! {
        pub static ref points: [[[u64; 10]; W as usize]; H as usize] = {
            let mut mt_init_hash: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0 as u64);
            let mut points_ = [[[0 as u64; 10]; W as usize]; H as usize];
            for y in 0..H as usize {
                for x in 0..W as usize {
                    for p in 1..10 {
                        points_[y][x][p] = mt_init_hash.gen();
                    }
                }
            }
            points_
        };
        pub static ref character: [[u64; W as usize]; H as usize] = {
            let mut mt_init_hash: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0 as u64);
            let mut character_ = [[0; W as usize]; H as usize];
            for y in 0..H as usize {
                for x in 0..W as usize {
                    character_[y][x] = mt_init_hash.gen();
                }
            }
            character_
        };
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct DistanceCoord {
    y_: i32,
    x_: i32,
    distance_: i32,
}
impl DistanceCoord {
    pub fn new(y: i32, x: i32, distance: i32) -> Self {
        Self {
            y_: y,
            x_: x,
            distance_: distance,
        }
    }
    pub fn newFromCoord(coord: &Coord) -> Self {
        Self {
            y_: coord.y_,
            x_: coord.x_,
            distance_: 0,
        }
    }
}
#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct WallMazeState {
    points_: [[i32; W as usize]; H as usize],
    walls_: [[i32; W as usize]; H as usize],
    turn_: i32,
    pub charcter_: Coord,
    pub game_score_: i32,
    pub evaluated_score_: ScoreType,
    pub first_action_: usize,
    pub hash_: u64,
}

impl WallMazeState {
    const dx: [i32; 4] = [1, -1, 0, 0];
    const dy: [i32; 4] = [0, 0, 1, -1];

    fn getDistanceToNearestPoint(&self) -> i32 {
        let mut que = VecDeque::new();
        que.push_back(DistanceCoord::newFromCoord(&self.charcter_));
        let mut check = [[false; W as usize]; H as usize];
        while !que.is_empty() {
            let tmp_cod = que.pop_front().unwrap();
            if self.points_[tmp_cod.y_ as usize][tmp_cod.x_ as usize] > 0 {
                return tmp_cod.distance_;
            }
            check[tmp_cod.y_ as usize][tmp_cod.x_ as usize] = true;
            for action in 0..4 {
                let ty = tmp_cod.y_ + Self::dy[action];
                let tx = tmp_cod.x_ + Self::dx[action];
                if ty >= 0
                    && ty < H
                    && tx >= 0
                    && tx < W
                    && self.walls_[ty as usize][tx as usize] == 0
                    && !check[ty as usize][tx as usize]
                {
                    que.push_back(DistanceCoord::new(ty, tx, tmp_cod.distance_ + 1));
                }
            }
        }
        H * W
    }

    fn init_hash(&mut self) {
        self.hash_ = 0;
        self.hash_ ^=
            zobrist_hash::character[self.charcter_.y_ as usize][self.charcter_.x_ as usize];
        for y in 0..H {
            for x in 0..W {
                let point = self.points_[y as usize][x as usize];
                if point > 0 {
                    self.hash_ ^= zobrist_hash::points[y as usize][x as usize][point as usize];
                }
            }
        }
    }

    pub fn new(seed: usize) -> Self {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        let mut maze_state = WallMazeState {
            points_: [[0 as i32; W as usize]; H as usize],
            walls_: [[0 as i32; W as usize]; H as usize],
            turn_: 0,
            charcter_: Coord { y_: 0, x_: 0 },
            game_score_: 0,
            evaluated_score_: 0,
            first_action_: 0,
            hash_: 0,
        };
        maze_state.charcter_.y_ = rng.gen_range(0..H);
        maze_state.charcter_.x_ = rng.gen_range(0..W);
        for y in (1..H).step_by(2) {
            for x in (1..W).step_by(2) {
                let mut ty = y;
                let mut tx = x;
                if ty == maze_state.charcter_.y_ && tx == maze_state.charcter_.x_ {
                    continue;
                }
                maze_state.walls_[ty as usize][tx as usize] = 1;
                let mut direction_size = 3;
                if y == 1 {
                    direction_size = 4;
                }
                let direction = rng.gen_range(0..direction_size);
                ty += Self::dy[direction as usize];
                tx += Self::dx[direction as usize];
                if ty == maze_state.charcter_.y_ && tx == maze_state.charcter_.x_ {
                    continue;
                }
                maze_state.walls_[ty as usize][tx as usize] = 1;
            }
        }

        for y in 0..H {
            for x in 0..W {
                if y == maze_state.charcter_.y_ && x == maze_state.charcter_.x_ {
                    continue;
                }
                maze_state.points_[y as usize][x as usize] = rng.gen_range(0..10);
            }
        }
        maze_state.init_hash();
        maze_state
    }

    pub fn isDone(&self) -> bool {
        return self.turn_ == END_TURN;
    }

    pub fn evaluateScore(&mut self) {
        self.evaluated_score_ = self.game_score_ * H * W - self.getDistanceToNearestPoint();
    }

    pub fn advance(&mut self, action: usize) {
        self.hash_ ^=
            zobrist_hash::character[self.charcter_.y_ as usize][self.charcter_.x_ as usize];

        self.charcter_.x_ += Self::dx[action];
        self.charcter_.y_ += Self::dy[action];
        let point = &mut self.points_[self.charcter_.y_ as usize][self.charcter_.x_ as usize];
        self.hash_ ^=
            zobrist_hash::character[self.charcter_.y_ as usize][self.charcter_.x_ as usize];

        if *point > 0 {
            self.hash_ ^= zobrist_hash::points[self.charcter_.y_ as usize]
                [self.charcter_.x_ as usize][*point as usize];
            self.game_score_ += *point;
            *point = 0;
        }
        self.turn_ += 1;
    }
    pub fn legalActions(&self) -> Vec<usize> {
        let mut actions = Vec::new();
        for action in 0..4 {
            let ty = self.charcter_.y_ + Self::dy[action];
            let tx = self.charcter_.x_ + Self::dx[action];
            if ty >= 0 && ty < H && tx >= 0 && tx < W && self.walls_[ty as usize][tx as usize] == 0
            {
                actions.push(action);
            }
        }
        actions
    }
    pub fn toString(&self) -> String {
        let mut s = String::new();
        s += format!("turn:\t{}\n", self.turn_).as_str();
        s += format!("score:\t{}\n", self.game_score_).as_str();
        for h in 0..H {
            for w in 0..W {
                if self.walls_[h as usize][w as usize] == 1 {
                    s.push('#');
                } else if self.charcter_.y_ == h && self.charcter_.x_ == w {
                    s.push('@');
                } else if self.points_[h as usize][w as usize] > 0 {
                    s += &self.points_[h as usize][w as usize].to_string();
                } else {
                    s.push('.');
                }
            }
            s.push('\n');
        }
        s
    }
}
impl Ord for WallMazeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluated_score_.cmp(&other.evaluated_score_)
    }
}
impl PartialOrd for WallMazeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
type State = WallMazeState;

fn randomAction(state: &State) -> usize {
    let mut rng = rand::thread_rng();

    let legal_actions = state.legalActions();
    return legal_actions[(rng.gen_range(0..=10) as usize % legal_actions.len())];
}

fn beamSearchAction(state: &State, beam_width: usize, beam_depth: usize) -> usize {
    let mut now_beam: BinaryHeap<State> = BinaryHeap::new();
    let mut best_state: State = State::new(11);
    now_beam.push(state.clone());
    let mut hash_check = HashSet::new();
    for t in 0..beam_depth {
        let mut next_beam: BinaryHeap<State> = BinaryHeap::new();
        for i in 0..beam_width {
            if now_beam.is_empty() {
                break;
            }
            let mut now_state = now_beam.pop().unwrap();
            let legal_actions = now_state.legalActions();
            for action in legal_actions {
                let mut next_state = now_state.clone();
                next_state.advance(action);
                if t >= 1 && hash_check.contains(&next_state.hash_) {
                    continue;
                }
                hash_check.insert(next_state.hash_);
                next_state.evaluateScore();
                if t == 0 {
                    next_state.first_action_ = action;
                }
                next_beam.push(next_state);
            }
        }
        now_beam = next_beam.clone();
        best_state = next_beam.pop().unwrap();
        if best_state.isDone() {
            break;
        }
    }
    best_state.first_action_
}

type AIFunction = fn(&State) -> usize;
type StringAIPair = (String, AIFunction);

fn testAiScore(ai: &StringAIPair, game_number: usize) {
    let mut rng = rand::thread_rng();
    let mut score_mean = 0.0;
    for i in 0..game_number {
        let mut state = State::new(rng.gen());
        while !state.isDone() {
            state.advance(ai.1(&state));
        }
        score_mean += state.game_score_ as f64;
    }
    score_mean /= game_number as f64;
    println!("Score of {}:\t{}", ai.0, score_mean);
}

fn main() {
    let f0: AIFunction = |state: &State| {
        return beamSearchAction(state, 100, END_TURN as usize);
    };
    let ai: StringAIPair = ("beamSearchAction".to_string(), f0);
    testAiScore(&ai, 100);
}
