use std::os::macos::raw::stat;

use montecalro::primitiveMontecarloAction;
use rand::{distributions::Standard, Rng};

const H: i32 = 5;
const W: i32 = 5;
const END_TURN: usize = 20;
const dstr: [&str; 4] = ["RIGHT", "LEFT", "DOWN", "UP"];

pub enum WinningStatus {
    FISRT,
    SECOND,
    DRAW,
    NONE,
}

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

    pub fn getWinningStatus(&self) -> WinningStatus {
        if self.isDone() {
            if self.characters_[0].game_score_ > self.characters_[1].game_score_ {
                return WinningStatus::FISRT;
            } else if self.characters_[0].game_score_ < self.characters_[1].game_score_ {
                return WinningStatus::SECOND;
            } else {
                return WinningStatus::DRAW;
            }
        } else {
            return WinningStatus::NONE;
        }
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

    pub fn getFirstPlayerScoreForWinRate(&self) -> f64 {
        match self.getWinningStatus() {
            WinningStatus::FISRT => return 1.0,
            WinningStatus::SECOND => return 0.0,
            _ => return 0.5,
        }
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

pub mod montecalro {
    use rand::Rng;

    use crate::randomAction;
    use crate::State;
    use crate::WinningStatus;
    use crate::INF;
    fn playout(state: &mut State) -> f64 {
        match state.getWinningStatus() {
            WinningStatus::FISRT => return 1.0,
            WinningStatus::SECOND => return 0.0,
            WinningStatus::DRAW => return 0.5,
            _ => {
                state.advance(randomAction(state, 0), randomAction(state, 1));
                return playout(state);
            }
        }
    }
    pub fn primitiveMontecarloAction(
        state: &State,
        player_id: usize,
        playout_number: usize,
    ) -> usize {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0);

        let my_legal_actions = state.legalActions(player_id);
        let opp_legal_actions = state.legalActions((player_id + 1) % 2);

        let mut best_action_index = 0;
        let mut best_value = -INF as f64;
        for i in 0..my_legal_actions.len() {
            let mut value = 0.0;
            for j in 0..playout_number {
                let mut next_state = state.clone();
                if player_id == 0 {
                    next_state.advance(
                        my_legal_actions[i],
                        opp_legal_actions[rng.gen_range(0..opp_legal_actions.len())],
                    )
                } else {
                    next_state.advance(
                        opp_legal_actions[rng.gen_range(0..opp_legal_actions.len())],
                        my_legal_actions[i],
                    )
                }
                let player0_win_rate = playout(&mut next_state);
                let win_rate = if player_id == 0 {
                    player0_win_rate
                } else {
                    1.0 - player0_win_rate
                };
                value += win_rate;
            }
            if value > best_value {
                best_value = value;
                best_action_index = i;
            }
        }
        my_legal_actions[best_action_index]
    }

    const C: f64 = 1.0;
    const EXPAND_THRESHOLD: usize = 5;
    struct Node {
        state_: State,
        w_: f64,
        pub child_nodeses_: Vec<Vec<Self>>,
        pub n_: f64,
    }
    impl Node {
        pub fn new(state: &State) -> Self {
            Self {
                state_: state.clone(),
                w_: 0.0,
                child_nodeses_: Vec::new(),
                n_: 0.0,
            }
        }
        pub fn evaluate(&mut self) -> f64 {
            if self.state_.isDone() {
                let mut value = 0.5;
                match self.state_.getWinningStatus() {
                    WinningStatus::FISRT => value = 1.0,
                    WinningStatus::SECOND => value = 0.0,
                    _ => value = 0.5,
                }
                self.w_ += value;
                self.n_ += 1.0;
                return value;
            }
            if self.child_nodeses_.is_empty() {
                let mut state_copy = self.state_.clone();
                let value = playout(&mut state_copy);
                self.w_ += value;
                self.n_ += 1.0;
                if self.n_ == EXPAND_THRESHOLD as f64 {
                    self.expand();
                }
                return value;
            } else {
                let value = self.nextChildNode().evaluate();
                self.w_ += value;
                self.n_ += 1.0;
                return value;
            }
        }

        fn expand(&mut self) {
            let legal_actions0 = self.state_.legalActions(0);
            let legal_actions1 = self.state_.legalActions(1);
            self.child_nodeses_.clear();
            for action0 in legal_actions0 {
                let mut add_nodes = Vec::new();
                for action1 in legal_actions1.iter() {
                    let mut add_state = Self::new(&self.state_);
                    add_state.state_.advance(action0, *action1);
                    add_nodes.push(add_state);
                }
                self.child_nodeses_.push(add_nodes);
            }
        }
        fn nextChildNode(&mut self) -> &mut Node {
            for i in 0..self.child_nodeses_.len() {
                for j in 0..self.child_nodeses_[i].len() {
                    if self.child_nodeses_[i][j].n_ == 0.0 {
                        return &mut self.child_nodeses_[i][j];
                    }
                }
            }

            let mut t = 0.0;
            for i in 0..self.child_nodeses_.len() {
                for j in 0..self.child_nodeses_[i].len() {
                    t += self.child_nodeses_[i][j].n_;
                }
            }

            let mut best_value = -INF as f64;
            let mut best_is = [0, 0];
            for i in 0..self.child_nodeses_.len() {
                let child_nodes = &self.child_nodeses_[i];
                let mut w = 0.0;
                let mut n = 0.0;
                for j in 0..child_nodes.len() {
                    w += child_nodes[j].w_;
                    n += child_nodes[j].n_;
                }
                let ucb1_value = w / n + (C as f64) * (2.0 * t.ln() / n).sqrt();
                if ucb1_value > best_value {
                    best_is[0] = i;
                    best_value = ucb1_value;
                }
            }
            for j in 0..self.child_nodeses_[0].len() {
                let mut w = 0.0;
                let mut n = 0.0;
                for i in 0..self.child_nodeses_.len() {
                    let child_node = &self.child_nodeses_[i][j];
                    w += child_node.w_;
                    n += child_node.n_;
                }
                w = 1.0 - w;
                let ucb1_value = w / n + (C as f64) * (2.0 * t.ln() / n).sqrt();
                if ucb1_value > best_value {
                    best_is[1] = j;
                    best_value = ucb1_value;
                }
            }
            best_value = -INF as f64;
            &mut self.child_nodeses_[best_is[0]][best_is[1]]
        }
    }
    pub fn ductAction(state: &State, player_id: usize, playout_number: usize) -> usize {
        let mut root_node = Node::new(state);
        root_node.expand();
        for i in 0..playout_number {
            root_node.evaluate();
        }
        let legal_actions = state.legalActions(player_id);
        let i_size = root_node.child_nodeses_.len();
        let j_size = root_node.child_nodeses_[0].len();
        if player_id == 0 {
            let mut best_action_searched_number = -1.0;
            let mut best_action_index = 0;
            for i in 0..i_size {
                let mut n = 0.0;
                for j in 0..j_size {
                    n += root_node.child_nodeses_[i][j].n_;
                }
                if n > best_action_searched_number {
                    best_action_index = i;
                    best_action_searched_number = n;
                }
            }
            return legal_actions[best_action_index];
        } else {
            let mut best_action_searched_number = -1.0;
            let mut best_j = 0;
            for j in 0..j_size {
                let mut n = 0.0;
                for i in 0..i_size {
                    n += root_node.child_nodeses_[i][j].n_;
                }
                if n > best_action_searched_number {
                    best_j = j;
                    best_action_searched_number = n;
                }
            }
            return legal_actions[best_j];
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
    const END_TURN_: usize = END_TURN * 2;
    pub fn new(base_state: &SimultaneousMazeState, player_id: usize) -> Self {
        let state = Self {
            points_: base_state.points_,
            turn_: base_state.turn_ * 2,
            characters_: if player_id == 0 {
                base_state.characters_.clone()
            } else {
                vec![base_state.characters_[1], base_state.characters_[0]]
            },
        };
        state
    }

    pub fn isDone(&self) -> bool {
        self.turn_ == Self::END_TURN_
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
                return WinningStatus::FISRT;
            } else if self.characters_[0].game_score_ < self.characters_[1].game_score_ {
                return WinningStatus::SECOND;
            } else {
                return WinningStatus::DRAW;
            }
        }
        WinningStatus::NONE
    }
}

type AlternateState = AlternateMazeState;

pub mod altanate_montecalro {
    use crate::AlternateState;
    use crate::State;
    use crate::WinningStatus;
    use crate::INF;
    use rand::Rng;

    fn randomAction(state: &AlternateState) -> usize {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0);
        let legal_actions = state.legalActions();
        let id = rng.gen_range(0..legal_actions.len());
        return legal_actions[id as usize];
    }

    fn playout(state: &mut AlternateState) -> f64 {
        match state.getWinningStatus() {
            WinningStatus::FISRT => return 1.0,
            WinningStatus::SECOND => return 0.0,
            WinningStatus::DRAW => return 0.5,
            _ => {
                state.advance(randomAction(state));
                return 1.0 - playout(state);
            }
        }
    }

    const C: f64 = 1.0;
    const EXPAND_THRESHOLD: usize = 10;
    struct Node {
        state_: AlternateState,
        w_: f64,
        pub child_nodes: Vec<Self>,
        pub n_: f64,
    }
    impl Node {
        pub fn new(state: &AlternateState) -> Self {
            Self {
                state_: state.clone(),
                w_: 0.0,
                child_nodes: Vec::new(),
                n_: 0.0,
            }
        }
        pub fn evaluate(&mut self) -> f64 {
            if self.state_.isDone() {
                let mut value = 0.5;
                match self.state_.getWinningStatus() {
                    WinningStatus::FISRT => value = 1.0,
                    WinningStatus::SECOND => value = 0.0,
                    _ => value = 0.5,
                }
                self.w_ += value;
                self.n_ += 1.0;
                return value;
            }
            if self.child_nodes.is_empty() {
                let mut state_copy = self.state_.clone();
                let value = playout(&mut state_copy);
                self.w_ += value;
                self.n_ += 1.0;
                if self.n_ == EXPAND_THRESHOLD as f64 {
                    self.expand();
                }
                return value;
            } else {
                let value = 1.0 - self.nextChildNode().evaluate();
                self.w_ += value;
                self.n_ += 1.0;
                return value;
            }
        }

        fn expand(&mut self) {
            let legal_actions = self.state_.legalActions();
            self.child_nodes.clear();
            for action in legal_actions {
                let mut add_state = Self::new(&self.state_);
                add_state.state_.advance(action);
                self.child_nodes.push(add_state);
            }
        }
        fn nextChildNode(&mut self) -> &mut Node {
            for i in 0..self.child_nodes.len() {
                if self.child_nodes[i].n_ == 0.0 {
                    return &mut self.child_nodes[i];
                }
            }
            let mut t = 0.0;
            for child_node in self.child_nodes.iter_mut() {
                t += child_node.n_;
            }
            let mut best_value = -INF as f64;
            let mut best_action_index = 0;
            for i in 0..self.child_nodes.len() {
                let child_node = &self.child_nodes[i];
                let ucb1_value = 1.0 - child_node.w_ / child_node.n_
                    + (C as f64) * (2.0 * t.ln() / child_node.n_).sqrt();
                if ucb1_value > best_value {
                    best_action_index = i;
                    best_value = ucb1_value;
                }
            }
            &mut self.child_nodes[best_action_index]
        }
    }
    pub fn mctsAction(base_state: &State, player_id: usize, playout_number: usize) -> usize {
        let state = AlternateState::new(base_state, player_id);
        let mut root_node = Node::new(&state);
        root_node.expand();
        for i in 0..playout_number {
            root_node.evaluate();
        }
        let legal_actions = state.legalActions();
        let mut best_action_searched_number = -1.0;
        let mut best_action_index = 0;
        assert_eq!(legal_actions.len(), root_node.child_nodes.len());
        for i in 0..legal_actions.len() {
            let n = root_node.child_nodes[i].n_;
            if n > best_action_searched_number {
                best_action_index = i;
                best_action_searched_number = n;
            }
        }
        legal_actions[best_action_index]
    }
}

type AIFunction = fn(&State, usize) -> usize;
type StringAIPair = (String, AIFunction);

fn testFirstPlayerWinRate(ais: [StringAIPair; 2], game_number: usize) {
    let mut first_player_win_rate = 0.0;
    for i in 0..game_number {
        let mut best_state = State::new(i);
        let mut state = best_state.clone();
        let first_ai = &ais[0];
        let second_ai = &ais[1];
        loop {
            state.advance(first_ai.1(&state, 0), second_ai.1(&state, 1));
            if state.isDone() {
                break;
            }
        }
        let mut win_rate_point = state.getFirstPlayerScoreForWinRate();

        if win_rate_point >= 0.0 {
            state.toString();
        }
        first_player_win_rate += win_rate_point;
        println!("i {} w {} ", i, first_player_win_rate / (i + 1) as f64);
    }
    first_player_win_rate /= (game_number) as f64;
    println!(
        "Winning rate of {} to {} :\t {}",
        ais[0].0, ais[1].0, first_player_win_rate
    );
}
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
    let f1: AIFunction =
        |state: &State, player_id: usize| return montecalro::ductAction(state, player_id, 50);
    let f0: AIFunction = |state: &State, player_id: usize| {
        return altanate_montecalro::mctsAction(state, player_id, 50);
    };
    let ais = [
        ("ductAction".to_string(), f1),
        ("mctsAction".to_string(), f0),
    ];
    testFirstPlayerWinRate(ais, 500);
}
