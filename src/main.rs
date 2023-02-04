use std::{fmt::Display, process::exit, fs::File, io::Read};

use cod::{InputManager, Key};
use random::{Source, Value};
use sarge::*;

macro_rules! max {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        std::cmp::max($x, max!( $($xs),+ ))
    };
}

type FlowResult<T> = Result<T, FlowError>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FlowError {
    InvalidCoords,
    EmptyTile,
    NoMoreColors,
    TileNotEmpty,
}

impl Display for FlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCoords => write!(f, "Invalid coordinates"),
            Self::EmptyTile => write!(f, "Can't drag empty square"),
            Self::NoMoreColors => write!(f, "No more available colors"),
            Self::TileNotEmpty => write!(f, "Tile is alreadx taken"),
        }
    }
}

fn escape<S: std::fmt::Display>(code: S) -> String {
    format!("{}[{}", 27 as char, code)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Color {
    #[default]
    Red,
    Yellow,
    Orange,
    Green,
    Blue,
    Purple,
    Pink,
    Gray,
}

impl Color {
    pub fn next(self) -> FlowResult<Self> {
        Ok(match self {
            Color::Red => Color::Orange,
            Color::Orange => Color::Blue,
            Color::Blue => Color::Pink,
            Color::Pink => Color::Yellow,
            Color::Yellow => Color::Green,
            Color::Green => Color::Purple,
            Color::Purple => Color::Gray,
            Color::Gray => Err(FlowError::NoMoreColors)?,
        })
    }

    pub fn colorize(&self, ch: char) -> String {
        escape(format!(
            "38;5;{}m{ch}",
            match self {
                Color::Red => 1,
                Color::Yellow => 3,
                Color::Orange => 173,
                Color::Green => 2,
                Color::Blue => 4,
                Color::Purple => 5,
                Color::Pink => 13,
                Color::Gray => 243,
            }
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tile {
    #[default]
    Empty,
    Source(Color),
    Flow(Color),
}

impl Tile {
    pub fn is_empty(self) -> bool {
        self == Tile::Empty
    }

    pub fn is_source(self) -> bool {
        matches!(self, Tile::Source(_))
    }

    pub fn is_flow(self) -> bool {
        matches!(self, Tile::Flow(_))
    }

    pub fn color(self) -> Option<Color> {
        match self {
            Tile::Flow(c) | Tile::Source(c) => Some(c),
            Tile::Empty => None,
        }
    }

    pub fn colorize(&self, ch: char) -> String {
        match self {
            Tile::Empty => ch.to_string(),
            Tile::Source(color) | Tile::Flow(color) => color.colorize(ch),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self {
                Self::Empty => "#".to_string(),
                Self::Source(color) => color.colorize('%'),
                Self::Flow(color) => color.colorize('*'),
            },
            escape("0m")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Direction {
    North,
    South,
    East,

    #[default]
    West,
}

impl TryFrom<Key> for Direction {
    type Error = String;

    fn try_from(value: Key) -> Result<Direction, String> {
        Ok(match value {
            Key::ArrowUp | Key::Char('w') => Self::North,
            Key::ArrowDown | Key::Char('s') => Self::South,
            Key::ArrowRight | Key::Char('d') => Self::East,
            Key::ArrowLeft | Key::Char('a') => Self::West,
            _ => return Err(format!("Invalid Key for direction: {value:?}")),
        })
    }
}

impl Value for Direction {
    fn read<S>(source: &mut S) -> Self
    where
        S: Source,
    {
        match source.read_u64() % 4 {
            0 => Self::North,
            1 => Self::South,
            2 => Self::East,
            3 => Self::West,
            _ => panic!("modulo stopped working, % 4 returned num>3"),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct FlowBoard {
    cols: [[Tile; 6]; 6],

    cursor_y: usize,
    cursor_x: usize,
    grabbed: bool,

    last: Color,
}

impl FlowBoard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_source(&mut self, y1: usize, x1: usize, y2: usize, x2: usize) -> FlowResult<()> {
        if max!(y1, x1, y2, x2) > 5 {
            return Err(FlowError::InvalidCoords);
        }

        if self.cols[y1][x1].is_empty() {
            self.cols[y1][x1] = Tile::Source(self.last);
        } else {
            return Err(FlowError::TileNotEmpty);
        }

        if self.cols[y2][x2].is_empty() {
            self.cols[y2][x2] = Tile::Source(self.last);
        } else {
            return Err(FlowError::TileNotEmpty);
        }

        self.last = self.last.next()?;
        Ok(())
    }

    pub fn get_yx(&self, y: usize, x: usize) -> FlowResult<Tile> {
        if max!(y, x) < 6 {
            return Ok(self.cols[y][x]);
        }

        Err(FlowError::InvalidCoords)
    }

    pub fn get(&self) -> Tile {
        self.cols[self.cursor_y][self.cursor_x]
    }

    pub fn set(&mut self, col: Color) -> FlowResult<()> {
        if self.get().is_source() && self.get().color().unwrap() == col {
            self.grabbed = false;
            return Ok(());
        } else if self.get().is_flow() && self.get().color() != Some(col) {
            self.clear_color(self.get().color().unwrap());
        } else if self.get().is_flow() && self.get().color() == Some(col) {
            self.clear_color(col);
            self.grabbed = false;
            return Ok(());
        } else if !self.get().is_empty() {
            return Err(FlowError::TileNotEmpty);
        }

        self.cols[self.cursor_y][self.cursor_x] = Tile::Flow(col);
        Ok(())
    }

    pub fn grab(&mut self) {
        if self.cols[self.cursor_y][self.cursor_x].is_empty() {
            return;
        }

        self.grabbed = !self.grabbed;
    }

    pub fn move_cursor(&mut self, dir: Direction) -> FlowResult<()> {
        match dir {
            Direction::East => {
                if self.cursor_x >= 5 {
                    return Ok(());
                }

                self.cursor_x += 1;
                if self.grabbed {
                    let col = match self.get_yx(self.cursor_y, self.cursor_x - 1)? {
                        Tile::Flow(c) => c,
                        Tile::Source(c) => c,
                        Tile::Empty => return Err(FlowError::EmptyTile),
                    };

                    self.set(col)?;
                }
            }
            Direction::West => {
                if self.cursor_x == 0 {
                    return Ok(());
                }

                self.cursor_x -= 1;
                if self.grabbed {
                    let col = match self.get_yx(self.cursor_y, self.cursor_x + 1)? {
                        Tile::Flow(c) => c,
                        Tile::Source(c) => c,
                        Tile::Empty => return Err(FlowError::EmptyTile),
                    };

                    self.set(col)?;
                }
            }
            Direction::South => {
                if self.cursor_y >= 5 {
                    return Ok(());
                }

                self.cursor_y += 1;
                if self.grabbed {
                    let col = match self.get_yx(self.cursor_y - 1, self.cursor_x)? {
                        Tile::Flow(c) => c,
                        Tile::Source(c) => c,
                        Tile::Empty => return Err(FlowError::EmptyTile),
                    };

                    self.set(col)?;
                }
            }
            Direction::North => {
                if self.cursor_y == 0 {
                    return Ok(());
                }

                self.cursor_y -= 1;
                if self.grabbed {
                    let col = match self.get_yx(self.cursor_y + 1, self.cursor_x)? {
                        Tile::Flow(c) => c,
                        Tile::Source(c) => c,
                        Tile::Empty => return Err(FlowError::EmptyTile),
                    };

                    self.set(col)?;
                }
            }
        }

        Ok(())
    }

    pub fn connected(&self, y: usize, x: usize) -> FlowResult<bool> {
        let color = match self.get_yx(y, x)? {
            Tile::Flow(c) => c,
            Tile::Source(c) => c,
            Tile::Empty => return Err(FlowError::EmptyTile),
        };

        let mut check = Vec::new();

        if y > 0 {
            check.push((y - 1, x));
        }

        if y < 6 {
            check.push((y + 1, x));
        }

        if x > 0 {
            check.push((y, x - 1));
        }

        if x < 6 {
            check.push((y, x + 1));
        }

        for (ny, nx) in check {
            if let Ok(tile) = self.get_yx(ny, nx) {
                if let Some(c) = tile.color() {
                    if c == color {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn clear_color(&mut self, col: Color) {
        self.cols.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|tile| {
                if tile.is_flow() && tile.color() == Some(col) {
                    *tile = Tile::Empty;
                }
            });
        });
    }

    pub fn is_solved(&self) -> FlowResult<bool> {
        for (y, col) in self.cols.iter().enumerate() {
            for (x, tile) in col.iter().enumerate() {
                if let Tile::Source(_) = tile {
                    if !self.connected(y, x)? {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }
}

impl Display for FlowBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (y, col) in self.cols.iter().enumerate() {
            for (x, tile) in col.iter().enumerate() {
                if self.cursor_y == y && self.cursor_x == x {
                    write!(f, "{}{}", tile.colorize('O'), escape("0m"))?;
                } else {
                    _ = write!(f, "{tile}");
                }
            }
            write!(f, "\n ")?;
        }

        Ok(())
    }
}

fn from_line(line: String) -> [(usize, usize); 2] {
    let mut a = (0, 0);
    let mut b = (0, 0);

    let coords: Vec<&str> = line.trim().split(' ').collect();
    if coords.len() != 4 {
        eprintln!("Invalid coordinate line: {}", line);
        exit(0);
    }

    a.0 = match coords[0].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    a.1 = match coords[1].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    b.0 = match coords[2].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    b.1 = match coords[3].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    [a, b]
}

fn from_file(filename: &String) -> Vec<[(usize, usize); 2]> {
    let mut sources = Vec::new();

    if let Ok(mut file) = File::open(filename) {
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        for line in data.lines() {
            sources.push(from_line(line.to_string()));
        }

    } else {
        eprintln!("Failed to open level file {filename}");
        exit(0);
    }

    sources
}

fn main() {
    let mut parser = ArgumentParser::new();
    let mut board = FlowBoard::new();
    let input = InputManager::new();

    let remainder = parser.parse().unwrap();
    if remainder.len() != 1 {
        eprintln!("Must give a level to play");
        return;
    }

    let filename = &remainder[0];

    for source in from_file(&filename) {
        board
            .set_source(source[0].0, source[0].1, source[1].0, source[1].1)
            .unwrap();
    }

    loop {
        cod::clear();
        cod::home();
        println!("{board}");
        cod::bot();

        if let Some(key) = input.poll() {
            match key {
                Key::Char(' ') => board.grab(),
                Key::Char('q') => return,
                Key::Escape => return,
                _ => {
                    if let Ok(dir) = key.try_into() {
                        if board.move_cursor(dir).is_err() {
                            board.grabbed = false;
                        }
                    }
                }
            }
        }

        if board.is_solved().unwrap() {
            cod::clear();
            cod::home();
            println!("{board}");
            cod::goto(1, 8);
            println!("  === VICTORY ===");
            return;
        }
    }
}
