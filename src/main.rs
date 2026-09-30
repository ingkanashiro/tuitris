use color_eyre::eyre::Result;
use colored::Colorize;
use rand::seq::IndexedRandom;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{
        Block,
        BorderType::Double,
        Paragraph,
        canvas::{Canvas, Rectangle},
    },
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct Mino {
    x: f64,
    y: f64,
    color: Color,
    from: Tetromino,
}

#[derive(Debug, Clone, Copy)]
enum Tetromino {
    I,
    O,
    J,
    L,
    T,
    S,
    Z,
}

struct TetrominoObject {
    minos: [Mino; 4],
    pos: [f64; 2],
    color: Color,
    from: Tetromino,
    state: u8,
}

struct AppState {
    period: u64,

    tetromino: TetrominoObject,
    next: [[Mino; 4]; 5],
    hold: [Mino; 4],

    has_hold: bool,
    can_hold: bool,

    minos: Vec<Mino>,
    game_over: bool,

    score: u64,
    level: u64,
    clears: u64,

    alert: u8,
}

impl AppState {
    fn update(&mut self) {
        self.period =
            (1000.0 * (0.8 - (self.level - 1) as f64 * 0.007).powf((self.level - 1) as f64)) as u64;

        if self.alert > 0 {
            self.alert -= 1;
        }

        if self.tetromino.minos.iter().any(|x| {
            (x.y + self.tetromino.pos[1] <= 0.0)
                || self.minos.iter().any(|m| {
                    m.x == x.x + self.tetromino.pos[0] && m.y == x.y + self.tetromino.pos[1] - 1.0
                })
        }) {
            for mino in &self.tetromino.minos {
                self.minos.push(Mino {
                    x: mino.x + self.tetromino.pos[0],
                    y: mino.y + self.tetromino.pos[1],
                    color: self.tetromino.color,
                    from: self.tetromino.from,
                });
            }

            // check if can't generate (lose condition)
            if self
                .minos
                .iter()
                .any(|mino| mino.y > 20.0 && mino.x <= 6.0 && mino.x >= 2.0)
            {
                // you lost
                self.game_over = true;
                return;
            }

            // generate new tetromino at top
            self.tetromino = new_tetromino(self.next[0]);

            self.next[0] = self.next[1];
            self.next[1] = self.next[2];
            self.next[2] = self.next[3];
            self.next[3] = self.next[4];
            self.next[4] = get_next_tetromino(get_random_tetromino());

            self.can_hold = true;
        } else {
            self.tetromino.pos[1] -= 1.0;
        }

        let mut row = 20.0;
        let mut rows_cleared = 0;
        let mut rows: Vec<f64> = vec![];

        while row >= 0.0 {
            let mut row_full = true;

            let mut col = 0.0;
            while col < 10.0 {
                if !self.minos.iter().any(|mino| mino.x == col && mino.y == row) {
                    row_full = false;
                    break;
                }

                col += 1.0;
            }

            if row_full {
                rows.push(row);

                rows_cleared += 1;
                self.clears += 1;

                if (self.clears) % 10 == 0 {
                    self.level += 1;
                    self.alert = 6;
                }
            }

            row -= 1.0;
        }

        self.clear_row(rows);

        // score
        self.score += (self.level + 1)
            * match rows_cleared {
                0 => 0,
                1 => 40,
                2 => 100,
                3 => 300,
                _ => 1200,
            }
    }

    fn clear_row(&mut self, rows: Vec<f64>) {
        for &row in rows.iter() {
            self.minos.retain(|mino| mino.y != row);

            for mino in self.minos.iter_mut() {
                if mino.y > row {
                    mino.y -= 1.0;
                }
            }
        }
    }

    fn hold(&mut self) {
        if !self.can_hold {
            return;
        }

        self.can_hold = false;

        if self.has_hold {
            let held = self.hold;

            self.hold = match self.tetromino.from {
                Tetromino::I => [
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 2.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::O => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::J => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::L => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::T => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::S => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::Z => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
            };

            self.tetromino = new_tetromino(held);
        } else {
            self.has_hold = true;

            self.hold = match self.tetromino.from {
                Tetromino::I => [
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 2.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::O => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::J => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::L => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::T => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::S => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
                Tetromino::Z => [
                    Mino {
                        x: 0.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 0.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: -1.0,
                        y: 1.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                    Mino {
                        x: 1.0,
                        y: 0.0,
                        color: self.tetromino.color,
                        from: self.tetromino.from,
                    },
                ],
            };
            self.tetromino = new_tetromino(self.next[0]);

            self.next[0] = self.next[1];
            self.next[1] = self.next[2];
            self.next[2] = self.next[3];
            self.next[3] = self.next[4];
            self.next[4] = get_next_tetromino(get_random_tetromino());
        }
    }
}

impl TetrominoObject {
    fn twist(&mut self, minos: &Vec<Mino>, flipped: bool) {
        // if flipped = true, then turn clockwise
        let fallback = self.state;

        match self.from {
            Tetromino::O => {
                return;
            }

            Tetromino::I => {
                self.state = (self.state + if flipped { 1 } else { 3 }) % 4;
                let rotated = match self.state {
                    0 => [
                        Mino {
                            x: -1.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 0.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 1.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 2.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                    ],

                    1 => [
                        Mino {
                            x: 0.0,
                            y: 2.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 0.0,
                            y: 1.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 0.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 0.0,
                            y: -1.0,
                            color: self.color,
                            from: self.from,
                        },
                    ],

                    2 => [
                        Mino {
                            x: -1.0,
                            y: 1.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 0.0,
                            y: 1.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 1.0,
                            y: 1.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 2.0,
                            y: 1.0,
                            color: self.color,
                            from: self.from,
                        },
                    ],

                    3 => [
                        Mino {
                            x: 1.0,
                            y: 2.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 1.0,
                            y: 1.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 1.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 1.0,
                            y: -1.0,
                            color: self.color,
                            from: self.from,
                        },
                    ],

                    _ => [
                        Mino {
                            x: -1.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 0.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 1.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                        Mino {
                            x: 2.0,
                            y: 0.0,
                            color: self.color,
                            from: self.from,
                        },
                    ],
                };

                // try for overlaps
                let delta = if flipped {
                    match self.state {
                        1 => [
                            // 0 -> R
                            [0.0, 0.0],
                            [-2.0, 0.0],
                            [1.0, 0.0],
                            [-2.0, -1.0],
                            [1.0, 2.0],
                        ],
                        2 => [
                            // R -> 2
                            [0.0, 0.0],
                            [-1.0, 0.0],
                            [2.0, 0.0],
                            [-1.0, 2.0],
                            [2.0, -1.0],
                        ],
                        3 => [
                            // 2 -> L
                            [0.0, 0.0],
                            [2.0, 0.0],
                            [-1.0, 0.0],
                            [2.0, 1.0],
                            [-1.0, 2.0],
                        ],
                        _ => [
                            // L -> 0
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [-2.0, 0.0],
                            [1.0, -2.0],
                            [-2.0, 1.0],
                        ],
                    }
                } else {
                    match self.state {
                        1 => [
                            // 2 -> R
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [-2.0, 0.0],
                            [1.0, -2.0],
                            [-2.0, 1.0],
                        ],
                        2 => [
                            // L -> 2
                            [0.0, 0.0],
                            [-2.0, 0.0],
                            [1.0, 0.0],
                            [-2.0, -1.0],
                            [1.0, 2.0],
                        ],
                        3 => [
                            // 0 -> L
                            [0.0, 0.0],
                            [-1.0, 0.0],
                            [2.0, 0.0],
                            [-1.0, 2.0],
                            [2.0, -1.0],
                        ],
                        _ => [
                            // R -> 0
                            [0.0, 0.0],
                            [-2.0, 0.0],
                            [-1.0, 0.0],
                            [2.0, 1.0],
                            [-1.0, -2.0],
                        ],
                    }
                };

                let mut d = 0;
                while d < 5 {
                    // try delta[d]
                    if rotated.iter().any(|mino| {
                        mino.y + self.pos[1] + delta[d][1] < 0.0
                            || mino.x + self.pos[0] + delta[d][0] < 0.0
                            || mino.x + self.pos[0] + delta[d][0] > 9.0
                            || minos.iter().any(|x| {
                                x.x == mino.x + self.pos[0] + delta[d][0]
                                    && x.y == mino.y + self.pos[1] + delta[d][1]
                            })
                    }) {
                        if d == 4 {
                            self.state = fallback;
                            return;
                        }

                        d += 1;
                    } else {
                        // found a possible rotation
                        self.pos[0] += delta[d][0];
                        self.pos[1] += delta[d][1];

                        break;
                    }
                }

                self.minos = rotated;
            }

            _ => {
                self.state = (self.state + if flipped { 1 } else { 3 }) % 4;
                let rotated: [Mino; 4] = if flipped {
                    std::array::from_fn(|i| {
                        let p = &self.minos[i];
                        Mino {
                            x: p.y,
                            y: -1.0 * p.x,
                            color: p.color,
                            from: p.from,
                        }
                    })
                } else {
                    std::array::from_fn(|i| {
                        let p = &self.minos[i];
                        Mino {
                            x: -1.0 * p.y,
                            y: p.x,
                            color: p.color,
                            from: p.from,
                        }
                    })
                };

                // try for overlaps
                let delta = if flipped {
                    match self.state {
                        1 => [
                            // 0 -> R
                            [0.0, 0.0],
                            [-1.0, 0.0],
                            [-1.0, 1.0],
                            [0.0, -2.0],
                            [-1.0, -2.0],
                        ],
                        2 => [
                            // R -> 2
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [1.0, -1.0],
                            [0.0, 2.0],
                            [1.0, 2.0],
                        ],
                        3 => [
                            // 2 -> L
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                            [0.0, -2.0],
                            [1.0, -2.0],
                        ],
                        _ => [
                            // L -> 0
                            [0.0, 0.0],
                            [-1.0, 0.0],
                            [-1.0, -1.0],
                            [0.0, 2.0],
                            [-1.0, 2.0],
                        ],
                    }
                } else {
                    match self.state {
                        1 => [
                            // 2 -> R
                            [0.0, 0.0],
                            [-1.0, 0.0],
                            [-1.0, 1.0],
                            [0.0, -2.0],
                            [-1.0, -2.0],
                        ],
                        2 => [
                            // L -> 2
                            [0.0, 0.0],
                            [-1.0, 0.0],
                            [-1.0, -1.0],
                            [0.0, 2.0],
                            [-1.0, 2.0],
                        ],
                        3 => [
                            // 0 -> L
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [1.0, 1.0],
                            [0.0, -2.0],
                            [1.0, -2.0],
                        ],
                        _ => [
                            // R -> 0
                            [0.0, 0.0],
                            [1.0, 0.0],
                            [1.0, -1.0],
                            [0.0, 2.0],
                            [1.0, 2.0],
                        ],
                    }
                };

                let mut d = 0;
                while d < 5 {
                    // try delta[d]
                    if rotated.iter().any(|mino| {
                        mino.y + self.pos[1] + delta[d][1] < 0.0
                            || mino.x + self.pos[0] + delta[d][0] < 0.0
                            || mino.x + self.pos[0] + delta[d][0] > 9.0
                            || minos.iter().any(|x| {
                                x.x == mino.x + self.pos[0] + delta[d][0]
                                    && x.y == mino.y + self.pos[1] + delta[d][1]
                            })
                    }) {
                        if d == 4 {
                            self.state = fallback;
                            return;
                        }

                        d += 1;
                    } else {
                        // found a possible rotation
                        self.pos[0] += delta[d][0];
                        self.pos[1] += delta[d][1];

                        break;
                    }
                }

                self.minos = rotated;
            }
        }
    }

    fn shift(&mut self, minos: &Vec<Mino>, dir: f64) {
        if !self.minos.iter().any(|x| {
            (x.x + self.pos[0] + dir < 0.0 || x.x + self.pos[0] + dir >= 10.0)
                || minos
                    .iter()
                    .any(|m| m.x == x.x + self.pos[0] + dir && m.y == x.y + self.pos[1])
        }) {
            self.pos[0] += dir;
        }
    }

    fn slam(&mut self, minos: &Vec<Mino>) {
        let mut delta = 0.0;

        while !self.minos.iter().any(|x| {
            (x.y + self.pos[1] + delta <= 0.0)
                || minos
                    .iter()
                    .any(|m| m.x == x.x + self.pos[0] && m.y == x.y + self.pos[1] + delta - 1.0)
        }) {
            delta -= 1.0;
        }

        self.pos[1] += delta;
    }
}

fn new_tetromino(minos: [Mino; 4]) -> TetrominoObject {
    let offset_x = 4.0;
    let offset_y = 20.0;

    TetrominoObject {
        pos: [offset_x, offset_y],
        minos: minos,
        color: minos[0].clone().color,
        from: minos[0].from,
        state: 0,
    }
}

fn get_next_tetromino(ttype: Tetromino) -> [Mino; 4] {
    let color = match ttype {
        Tetromino::I => Color::Cyan,
        Tetromino::O => Color::Yellow,
        Tetromino::J => Color::Blue,
        Tetromino::L => Color::LightRed,
        Tetromino::T => Color::Magenta,
        Tetromino::S => Color::Green,
        Tetromino::Z => Color::Red,
    };

    let minos_array = match ttype {
        Tetromino::I => [
            Mino {
                x: -1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 0.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 2.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
        ],
        Tetromino::O => [
            Mino {
                x: 0.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 0.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
        ],
        Tetromino::J => [
            Mino {
                x: 0.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: -1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: -1.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
        ],
        Tetromino::L => [
            Mino {
                x: 0.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: -1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
        ],
        Tetromino::T => [
            Mino {
                x: 0.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: -1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 0.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
        ],
        Tetromino::S => [
            Mino {
                x: 0.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: -1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 0.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
        ],
        Tetromino::Z => [
            Mino {
                x: 0.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 0.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: -1.0,
                y: 1.0,
                color: color,
                from: ttype,
            },
            Mino {
                x: 1.0,
                y: 0.0,
                color: color,
                from: ttype,
            },
        ],
    };

    return minos_array;
}

fn get_random_tetromino() -> Tetromino {
    let mut rng = rand::rng();
    let types = [
        Tetromino::I,
        Tetromino::O,
        Tetromino::J,
        Tetromino::L,
        Tetromino::T,
        Tetromino::S,
        Tetromino::Z,
    ];

    let chosen_type = types.choose(&mut rng).unwrap();

    return chosen_type.clone();
}

fn main() -> Result<()> {
    color_eyre::install()?;

    println!(
        "Running {} on terminal...",
        Colorize::bold("tuitris v1.0").blue()
    );

    let state = &mut AppState {
        period: 20,
        minos: vec![],
        tetromino: new_tetromino(get_next_tetromino(get_random_tetromino())),
        next: [
            get_next_tetromino(get_random_tetromino()),
            get_next_tetromino(get_random_tetromino()),
            get_next_tetromino(get_random_tetromino()),
            get_next_tetromino(get_random_tetromino()),
            get_next_tetromino(get_random_tetromino()),
        ],
        hold: get_next_tetromino(Tetromino::I),
        has_hold: false,
        can_hold: true,
        game_over: false,
        score: 0,
        level: 1,
        clears: 0,
        alert: 0,
    };

    let terminal = ratatui::init();
    let result = run(terminal, state);

    ratatui::restore();

    println!(
        "\nGame finished after reaching {} with {} and {}.\nYou can run {} to play again.\n",
        Colorize::bold(format!("level {}", state.level).as_str()).cyan(),
        Colorize::bold(format!("{} points", state.score).as_str()).yellow(),
        Colorize::bold(format!("{} rows cleared", state.clears).as_str()).magenta(),
        Colorize::bold("tetris").blue()
    );

    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    let mut tick_rate; // Controls game speed (lower = faster)
    let mut last_tick = Instant::now();

    loop {
        tick_rate = Duration::from_millis(app_state.period);
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Rendering
        terminal.draw(|f| render(f, app_state))?;

        // Input
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    event::KeyCode::Esc => {
                        break;
                    }

                    event::KeyCode::Left => {
                        app_state.tetromino.shift(&app_state.minos, -1.0);
                    }

                    event::KeyCode::Right => {
                        app_state.tetromino.shift(&app_state.minos, 1.0);
                    }

                    event::KeyCode::Char('z') => {
                        app_state.tetromino.twist(&app_state.minos, false);
                    }

                    event::KeyCode::Char('x') => {
                        app_state.tetromino.twist(&app_state.minos, true);
                    }

                    event::KeyCode::Up => {
                        app_state.tetromino.twist(&app_state.minos, true);
                    }

                    event::KeyCode::Down => {
                        tick_rate = if key.kind == KeyEventKind::Press {
                            if 120 < app_state.period {
                                Duration::from_millis(80)
                            } else {
                                Duration::from_millis(app_state.period)
                            }
                        } else {
                            Duration::from_millis(app_state.period)
                        };
                    }

                    event::KeyCode::Char(' ') => {
                        app_state.tetromino.slam(&app_state.minos);
                        tick_rate = Duration::from_millis(0);
                    }

                    event::KeyCode::Char('c') => {
                        app_state.hold();
                    }

                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app_state.update();
            last_tick = Instant::now();
        }

        if app_state.game_over {
            break;
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let main_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(28),
            Constraint::Fill(1),
        ])
        .split(frame.area());

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(16),
            Constraint::Length(35),
            Constraint::Length(16),
            Constraint::Fill(1),
        ])
        .split(main_area[1]);

    let hold_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7)])
        .split(center[1]);

    let hold_panel = Block::bordered()
        .fg(if app_state.can_hold {
            Color::Magenta
        } else {
            Color::DarkGray
        })
        .border_type(Double)
        .title_top(Line::from("[ HOLD (C) ]").centered().bold());

    let side_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(20),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(center[3]);

    let board = Block::bordered()
        .fg(if &app_state.alert % 2 == 1 {
            Color::LightCyan
        } else {
            Color::White
        })
        .border_type(Double);

    let score_border = Block::bordered()
        .fg(Color::Yellow)
        .border_type(Double)
        .title_top(Line::from("[ SCORE ]").centered().bold());

    let level_border = Block::bordered()
        .fg(if &app_state.alert % 2 == 1 {
            Color::LightCyan
        } else {
            Color::Cyan
        })
        .border_type(Double)
        .title_top(Line::from("[ LEVEL ]").centered().bold());

    let nextup_border = Block::bordered()
        .fg(Color::Gray)
        .border_type(Double)
        .title_top(Line::from("[ NEXT UP ]").centered().bold());

    let nextup_display = Canvas::default()
        .block(nextup_border)
        .marker(Marker::Braille)
        .x_bounds([0.0, 6.0])
        .y_bounds([0.0, 21.0])
        .paint(|ctx| {
            let mut next_index = 0;
            while next_index < 4 {
                for mino in app_state.next[next_index].iter() {
                    ctx.draw(&Rectangle {
                        x: mino.x + 2.0,
                        y: mino.y + 17.0 - (5.0 * next_index as f64),
                        width: 1.0,
                        height: 1.0,
                        color: mino.color,
                    })
                }

                next_index += 1;
            }
        });

    let hold_display = Canvas::default()
        .block(hold_panel)
        .marker(Marker::Braille)
        .x_bounds([0.0, 6.0])
        .y_bounds([0.0, 6.0])
        .paint(|ctx| {
            if app_state.has_hold {
                for mino in app_state.hold.iter() {
                    ctx.draw(&Rectangle {
                        x: mino.x + 2.0,
                        y: mino.y + 2.0,
                        width: 1.0,
                        height: 1.0,
                        color: mino.color,
                    })
                }
            }
        });

    let mut delta = 0.0;

    while !&app_state.tetromino.minos.iter().any(|x| {
        (x.y + &app_state.tetromino.pos[1] + delta <= 0.0)
            || app_state.minos.iter().any(|m| {
                m.x == x.x + app_state.tetromino.pos[0]
                    && m.y == x.y + app_state.tetromino.pos[1] + delta - 1.0
            })
    }) {
        delta -= 1.0;
    }

    let board_display = Canvas::default()
        .block(board)
        .marker(Marker::Braille)
        .x_bounds([0.0, 10.0])
        .y_bounds([0.0, 20.0])
        .paint(|ctx| {
            for mino in &app_state.tetromino.minos {
                ctx.draw(&Rectangle {
                    x: mino.x + &app_state.tetromino.pos[0],
                    y: mino.y + &app_state.tetromino.pos[1] + delta,
                    width: 1.0,
                    height: 1.0,
                    color: Color::DarkGray,
                });

                ctx.draw(&Rectangle {
                    x: mino.x + &app_state.tetromino.pos[0],
                    y: mino.y + &app_state.tetromino.pos[1],
                    width: 1.0,
                    height: 1.0,
                    color: mino.color,
                });
            }

            for mino in &app_state.minos {
                ctx.draw(&Rectangle {
                    x: mino.x,
                    y: mino.y,
                    width: 1.0,
                    height: 1.0,
                    color: mino.color,
                });

                let mut fill_h = mino.y;
                while fill_h <= mino.y + 1.0 {
                    ctx.draw(&ratatui::widgets::canvas::Line {
                        x1: mino.x,
                        y1: fill_h,
                        x2: mino.x + 1.0,
                        y2: fill_h,
                        color: mino.color,
                    });

                    fill_h += 0.25;
                }
            }
        });

    let score_display =
        Paragraph::new(Line::from(format!("{}", &app_state.score)).centered()).block(score_border);

    let level_display = Paragraph::new(if &app_state.alert % 2 == 1 {
        Line::from("LEVEL UP!").centered().bold()
    } else {
        Line::from(format!("{}", &app_state.level)).centered()
    })
    .block(level_border);

    frame.render_widget(hold_display, hold_area[0]);
    frame.render_widget(board_display, center[2]);
    frame.render_widget(nextup_display, side_panel[0]);
    frame.render_widget(score_display, side_panel[1]);
    frame.render_widget(level_display, side_panel[2]);
}
