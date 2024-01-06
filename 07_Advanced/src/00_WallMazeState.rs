use std::time;

use rand;
use rand::prelude::*;

#[derive(Debug, Default)]
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

const H: i32 = 5;
const W: i32 = 5;
const END_TURN: i32 = 4;

#[derive(Debug, Default)]
struct WallMazeState {
    points_: [[i32; W as usize]; H as usize],
    walls_: [[i32; W as usize]; H as usize],
    turn_: i32,
    pub charcter_: Coord,
    pub game_score_: i32,
    pub evaluated_score_: ScoreType,
    pub fist_action_: usize,
}
impl WallMazeState {
    const dx: [i32; 4] = [1, -1, 0, 0];
    const dy: [i32; 4] = [0, 0, 1, -1];
    pub fn new(seed: usize) -> Self {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);
        let mut maze_state = WallMazeState {
            points_: [[0 as i32; W as usize]; H as usize],
            walls_: [[0 as i32; W as usize]; H as usize],
            turn_: 0,
            charcter_: Coord { y_: 0, x_: 0 },
            game_score_: 0,
            evaluated_score_: 0,
            fist_action_: 0,
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
        maze_state
    }

    pub fn isDone(&self) -> bool {
        return self.turn_ == END_TURN;
    }

    pub fn advance(&mut self, action: usize) {
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
            if ty >= 0 && ty < H && tx >= 0 && tx < W && self.walls_[ty as usize][tx as usize] == 0
            {
                actions.push(action);
            }
        }
        actions
    }
    pub fn toString(&self) -> String {
        let mut s = String::new();
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
type State = WallMazeState;

fn randomAction(state: &State) -> usize {
    let mut rng = rand::thread_rng();

    let legal_actions = state.legalActions();
    return legal_actions[(rng.gen_range(0..=10) as usize % legal_actions.len())];
}
fn playGame(seed: usize) {
    let mut state = State::new(seed);
    println!("{}", state.toString());
    while !state.isDone() {
        state.advance(randomAction(&state));
        println!("{}", state.toString());
    }
}
fn main() {
    playGame(0);
}
