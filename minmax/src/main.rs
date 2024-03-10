use std::fmt;

const NUM_OF_COINS_IN_GAME: u8 = 5;
const MAX_COINS_TO_TAKE: u8 = 2;

struct State {
    coins: u8,
    is_player_turn: bool,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "State {{ coins: {}, is_player_turn: {} }}\n",
            self.coins, self.is_player_turn
        )
    }
}

impl State {
    fn new() -> Self {
        State {
            coins: NUM_OF_COINS_IN_GAME,
            is_player_turn: true,
        }
    }

    fn take_turn(&mut self, num_of_coins: u8) {
        self.coins -= num_of_coins;
        self.is_player_turn = !self.is_player_turn;
    }

    fn is_game_over(&self) -> bool {
        self.coins == 0
    }

    fn copy(&self) -> Self {
        State {
            coins: self.coins,
            is_player_turn: self.is_player_turn,
        }
    }
}

fn solver() -> Vec<State> {
    let mut solutions: Vec<State> = Vec::new();
    let state = State::new();
    let mut stack: Vec<State> = Vec::new();
    stack.push(state);

    while !stack.is_empty() {
        let state = stack.pop().unwrap();
        if state.is_game_over() {
            solutions.push(state);
        }
    }
}

fn minmax(state: &State) -> u8 {
    if state.is_game_over() {
        return state.coins;
    }
    if state.is_player_turn {
        max_value(state)
    } else {
        min_value(state)
    }
}

fn max_value(state: &State) -> u8 {
    if state.is_game_over() {
        return state.coins;
    }

    let mut max_val = 0;
    for coins in 1..=MAX_COINS_TO_TAKE {
        if state.coins >= coins {
            let mut next_state = state.copy();
            next_state.take_turn(coins);
            let val = min_value(&next_state);
            max_val = max_val.max(val);
        }
    }
    max_val
}

fn min_value(state: &State) -> u8 {
    if state.is_game_over() {
        return state.coins;
    }

    let mut min_val = u8::MAX;
    for coins in 1..=MAX_COINS_TO_TAKE {
        if state.coins >= coins {
            let mut next_state = state.copy();
            next_state.take_turn(coins);
            let val = max_value(&next_state);
            min_val = min_val.min(val);
        }
    }
    min_val
}

fn main() {
    println!("Hello, world!");
}
/*
```
 (START)               5----------
                      /           \
    A - max          4              3
                    / \            / \
    B - min   -----3   2          2   1
             /    /   / \        / \   \
 A - max    2    1   1   0      1   0   0
           /|    |   |          |
 B - min  1 0    0   0          0
          |
 A - max  0

Napisz zmiany v dla tego grafu
```
```
                    4------   
                  /   |    \
                 /    |     \
                /     |      \
     --------- 3      2       1 
    / | \             | \     |
   /  |  \            1  0    0 
  2   1   0           |  
 / \  |               0  
1   0 0 
|
0
 ```
 
*/