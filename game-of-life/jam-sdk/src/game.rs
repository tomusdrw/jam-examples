use alloc::{vec, vec::Vec};

/// Quite inefficient way of storing the board in memory (intentionally)
pub struct Game {
    size: usize,
    board: Vec<Vec<bool>>,
}

pub struct XY(usize, usize);

pub enum Mutation {
  On(XY),
  Off(XY),
}

impl Game {
    pub fn empty(size: u32) -> Self {
        let size = size as usize;
        let mut board = vec![];
        for _i in 0..size {
            board.push(vec![false; size]);
        }

        Self { size, board }
    }

    pub fn new(size: u32, flat: &[u8]) -> Self {
        let mut game = Self::empty(size);
        let size = size as usize;
        assert_eq!(size * size, flat.len(), "Invalid board size");
        // initialize board
        for x in 0..size {
            for y in 0..size {
                game.board[x][y] = flat[x * size + y] > 0;
            }
        }
        game
    }

    pub fn export(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.size * self.size);
        for x in &self.board {
            for &b in x {
                result.push(if b { 0xff } else { 0 });
            }
        }
        result
    }

    fn count_neighbours(&self, pos: XY) -> u32 {
        [
            (1i32, -1i32),
            (1, 0),
            (1, 1),
            (0, -1),
            (0, 1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ].iter()
            .map(|&(mod_x, mod_y)| {

                let n_x = (pos.0 as i32 + mod_x) % self.size as i32;
                let n_y = (pos.1 as i32 + mod_y) % self.size as i32;

                (n_x as usize, n_y as usize)
            })
        .filter(|&(n_x, n_y)| {
            let is_alive = self.board[n_x][n_y];
            is_alive
        })
        .count() as u32
    }

    pub fn mutate(&mut self, mutations: &[Mutation]) {
        for m in mutations {
            match *m {
                Mutation::On(XY(x, y)) => self.board[x][y] = true,
                Mutation::Off(XY(x, y)) => self.board[x][y] = false,
            }
        }
    }

    // TODO [ToDr] to avoid leaks with LeakingAllocator maybe better to just maintain
    // a second copy of the board
    pub fn next_step(&self) -> Vec<Mutation> {
        let mut mutations = vec![];
        for x in 0..self.size {
            for y in 0..self.size {
                let current_is_alive = self.board[x][y];
                let no_of_neighbours = self.count_neighbours(XY(x, y));

                if current_is_alive {
                    if no_of_neighbours != 2 && no_of_neighbours != 3 {
                        mutations.push(Mutation::Off(XY(x, y)));
                    }
                } else {
                    if no_of_neighbours == 3 {
                        mutations.push(Mutation::On(XY(x, y)))
                    }
                }
            }
        }
        mutations
    }
}
