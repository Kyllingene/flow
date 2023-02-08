use std::{cmp::max, fmt::Display, fs::File, io::Read, process::exit};

use cod::{InputManager, Key};
use random::{Source, Value};
use sarge::*;

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
    cols: Vec<Vec<Tile>>,
    size_y: usize,
    size_x: usize,

    cursor_y: usize,
    cursor_x: usize,
    grabbed: bool,

    last: Color,
}

impl FlowBoard {
    pub fn new(x: usize, y: usize) -> Self {
        let mut row = Vec::with_capacity(x);
        for _ in 0..x {
            row.push(Tile::Empty);
        }

        let mut cols = Vec::with_capacity(y);
        for _ in 0..y {
            cols.push(row.clone());
        }

        Self {
            size_x: x,
            size_y: y,
            cols,
            ..Default::default()
        }
    }

    pub fn set_source(&mut self, y1: usize, x1: usize, y2: usize, x2: usize) -> FlowResult<()> {
        if max(y1, y2) > self.size_y || max(x1, x2) > self.size_x {
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
        if y < self.size_y && x < self.size_x {
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
                if self.cursor_x >= self.size_x - 1 {
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
                if self.cursor_y >= self.size_y - 1 {
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
        write!(f, "+{:-<1$}+", "", self.size_x)?;
        for (y, col) in self.cols.iter().enumerate() {
            write!(f, "\n |")?;
            for (x, tile) in col.iter().enumerate() {
                if self.cursor_y == y && self.cursor_x == x {
                    write!(f, "{}{}", tile.colorize('O'), escape("0m"))?;
                } else {
                    write!(f, "{tile}")?;
                }
            }
            write!(f, "|")?;
        }

        write!(f, "\n +{:-<1$}+", "", self.size_x)?;

        Ok(())
    }
}

fn from_line(line: String) -> [(usize, usize); 2] {
    let mut a = (0, 0);
    let mut b = (0, 0);

    let coords: Vec<&str> = line.trim().split(' ').collect();
    if coords.len() != 4 {
        eprintln!("Invalid coordinate line: {line}");
        exit(0);
    }

    a.1 = match coords[0].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    a.0 = match coords[1].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    b.1 = match coords[2].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    b.0 = match coords[3].parse() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("Invalid coordinate: {}", coords[0]);
            exit(0);
        }
    };

    [a, b]
}

fn from_file(filename: &String) -> (usize, usize, Vec<[(usize, usize); 2]>) {
    let mut sources = Vec::new();
    let size_x;
    let size_y;

    if let Ok(mut file) = File::open(filename) {
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let mut lines = data.lines();
        let sizes = lines.next().unwrap_or_else(|| {
            eprintln!("Invalid level: empty");
            exit(1);
        });

        let split = sizes.split(' ').collect::<Vec<&str>>();
        if split.len() != 2 {
            eprintln!("Invalid level: invalid size line: {sizes}");
            exit(1);
        }

        size_x = match split[0].parse() {
            Ok(i) => i,
            Err(_) => {
                eprintln!("Invalid x size: {}", split[0]);
                exit(0);
            }
        };

        size_y = match split[1].parse() {
            Ok(i) => i,
            Err(_) => {
                eprintln!("Invalid x size: {}", split[1]);
                exit(0);
            }
        };

        for line in lines {
            if !line.is_empty() {
                sources.push(from_line(line.to_string()));
            }
        }
    } else {
        eprintln!("Failed to open level file {filename}");
        exit(0);
    }

    (size_x, size_y, sources)
}

fn main() {
    let mut parser = ArgumentParser::new();
    let input = InputManager::new();

    let remainder = parser.parse().unwrap();
    if remainder.len() != 1 {
        eprintln!("Must give a level to play");
        return;
    }

    let filename = &remainder[0];

    let (size_x, size_y, sources) = from_file(filename);
    let mut board = FlowBoard::new(size_x, size_y);

    for source in sources {
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
            cod::goto(0, board.size_y as u32 + 2);
            println!("=== VICTORY ===");
            return;
        }
    }
}
