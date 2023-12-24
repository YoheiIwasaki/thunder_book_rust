use std::collections::BinaryHeap;

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

type ScoreType = i64;
const INF: ScoreType = 1000000000;
const H: i32 = 3;
const W: i32 = 4;
const END_TURN: i32 = 4;
const INVALID_ACTION: usize = 100000;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct MazeState {
    points_: [[i32; W as usize]; H as usize],
    turn_: i32,
    pub charcter_: Coord,
    pub game_score_: i32,
    pub evaluated_score_: ScoreType,
    pub first_action_: usize,
}
impl MazeState {
    const dx: [i32; 4] = [1, -1, 0, 0];
    const dy: [i32; 4] = [0, 0, 1, -1];
    pub fn new(seed: usize) -> Self {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        let mut maze_state = MazeState {
            points_: [[0 as i32; W as usize]; H as usize],
            turn_: 0,
            charcter_: Coord { y_: 0, x_: 0 },
            game_score_: 0,
            evaluated_score_: 0,
            first_action_: 10,
        };
        maze_state.charcter_.y_ = rng.gen_range(0..=10) % H;
        maze_state.charcter_.x_ = rng.gen_range(0..=10) % W;
        for y in 0..H {
            for x in 0..W {
                if y == maze_state.charcter_.y_ && x == maze_state.charcter_.x_ {
                    continue;
                }
                maze_state.points_[y as usize][x as usize] = rng.gen_range(1..=9);
            }
        }
        maze_state
    }

    pub fn isDone(&self) -> bool {
        return self.turn_ == END_TURN;
    }

    pub fn evaluateScore(&mut self) {
        self.evaluated_score_ = self.game_score_ as ScoreType;
    }

    pub fn advance(&mut self, action: usize) {
        if action == INVALID_ACTION {
            self.turn_ += 1;
            return;
        }
        self.charcter_.x_ += Self::dx[action];
        self.charcter_.y_ += Self::dy[action];
        let point = &mut self.points_[self.charcter_.y_ as usize][self.charcter_.x_ as usize];
        if *point > 0 {
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
            if ty >= 0 && ty < H && tx >= 0 && tx < W {
                actions.push(action);
            }
        }
        actions
    }
    pub fn toString(&self) -> String {
        let mut s = String::new();
        s += "turn:\t";
        s += &self.turn_.to_string();
        s += "\n";
        s += "score:\t";
        s += &self.game_score_.to_string();
        s += "\n";
        for h in 0..H {
            for w in 0..W {
                if self.charcter_.y_ == h && self.charcter_.x_ == w {
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

impl Ord for MazeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.evaluated_score_.cmp(&other.evaluated_score_)
    }
}
impl PartialOrd for MazeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

type State = MazeState;

fn randomAction(state: &State) -> usize {
    let mut rng = rand::thread_rng();

    let legal_actions = state.legalActions();
    return legal_actions[(rng.gen_range(0..=10) as usize % legal_actions.len())];
}

fn greedyAction(state: &State) -> usize {
    let legal_actions = state.legalActions();
    let mut best_score: ScoreType = -1;
    let mut best_action = 10;
    for action in legal_actions {
        let mut now_state = state.clone();
        now_state.advance(action);
        now_state.evaluateScore();
        if now_state.evaluated_score_ > best_score {
            best_score = now_state.evaluated_score_;
            best_action = action;
        }
    }
    best_action
}

fn beamSearchAction(state: &State, beam_width: usize, beam_depth: usize) -> usize {
    let mut now_beam: BinaryHeap<State> = BinaryHeap::new();
    let mut best_state: State = State::new(11);
    now_beam.push(state.clone());
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

fn chokudaiSearchAction(
    state: &State,
    beam_width: usize,
    beam_depth: usize,
    beam_number: usize,
) -> usize {
    let mut beam = Vec::new();
    for t in 0..beam_depth + 1 {
        beam.push(BinaryHeap::<State>::new());
    }
    beam[0].push(state.clone());
    for cnt in 0..beam_number {
        for t in 0..beam_depth {
            for i in 0..beam_width {
                if beam[t].is_empty() {
                    break;
                }
                let mut now_state = beam[t].peek().unwrap().clone();
                if now_state.isDone() {
                    break;
                }
                beam[t].pop();
                let legal_actions = now_state.legalActions();
                for action in legal_actions {
                    let mut next_state = now_state.clone();
                    next_state.advance(action);
                    next_state.evaluateScore();
                    if t == 0 {
                        next_state.first_action_ = action;
                    }
                    beam[t + 1].push(next_state);
                }
            }
        }
    }
    for t in 0..=beam_depth {
        let now_beam = &beam[beam_depth - t];
        if !now_beam.is_empty() {
            return now_beam.peek().unwrap().first_action_;
        }
    }
    INVALID_ACTION
}

fn playGame(seed: usize) {
    let mut state = State::new(seed);
    println!("{}", state.toString());
    while !state.isDone() {
        // state.advance(randomAction(&state));
        state.advance(greedyAction(&state));
        println!("{}", state.toString());
    }
}

fn testAiScore(game_number: usize) {
    let mut rng = rand::thread_rng();
    let mut score_mean = 0.0;
    for i in 0..game_number {
        let mut state = State::new(rng.gen_range(0..1000) as usize);
        while !state.isDone() {
            state.advance(chokudaiSearchAction(&state, 1, END_TURN as usize, 2));
            // state.advance(beamSearchAction(&state, 2, END_TURN as usize));
            // state.advance(greedyAction(&state));
        }
        score_mean += state.game_score_ as f64;
    }
    score_mean /= game_number as f64;
    println!("Score:\t {}", score_mean);
}

fn main() {
    testAiScore(100);
}
