use std::{self, env, fs::File, io::{Read, Seek, SeekFrom, Write}, ops};
use getch::Getch;

#[derive(PartialEq, Copy, Clone, Debug)]
struct Coord{
	x: isize,
	y: isize,
}
impl ops::Add for Coord{
	type Output = Coord;

	fn add(self, rhs: Self) -> Self::Output {Self{
		x: self.x + rhs.x,
		y: self.y + rhs.y
	}}
}
impl ops::AddAssign for Coord{
	fn add_assign(&mut self, rhs: Self) {*self = *self+rhs}
}
impl From<(isize, isize)> for Coord{
	fn from(value: (isize, isize)) -> Self {
		Self{x:value.0, y:value.1}
	}
}

struct Board{
	width: usize,
	height: usize,
	board: Vec<Square>,
	selected_square: Coord,
	selected_flow: Option<u32>,
}
impl Board{
	fn get(&self, c: Coord) -> &Square { &self.board[c.y as usize*self.width+c.x as usize] }
	fn get_mut(&mut self, c: Coord) -> &mut Square { &mut self.board[c.y as usize*self.width+c.x as usize] }
	fn set(&mut self, c: Coord, s:Square) { self.board[c.y as usize*self.width+c.x as usize] = s; }
	fn get_other_entry(&self, c: Coord) -> Option<Coord> {
		if let Square::Entry{flow, ..} = self.get(c){
			for y in 0..self.height as isize{
				for x in 0..self.width as isize{
					if
						c != (x, y).into() &&
						let Square::Entry{flow: other_flow, ..} = self.get((x, y).into()) &&
						flow == other_flow
					{return Some((x, y).into())}
				}
			}
		}
		return None;
	}
	fn display(&self){
		std::io::stdout().write_all("\u{250c}".as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		std::io::stdout().write_all("\u{2500}".repeat(self.width*2+1).as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		std::io::stdout().write_all("\u{2510}".as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		std::io::stdout().write_all(if let Some(flow) = self.selected_flow{
			String::from_utf8(vec![flow as u8 + b'A']).unwrap()
		}else{"-".to_string()}.as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		std::io::stdout().write_all("\n".as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		for y in 0..self.height as isize{
			std::io::stdout().write_all("\u{2502}".as_bytes()).map_err(|err| println!("{:?}", err)).ok();
			for x in 0..self.width as isize{
				std::io::stdout().write_all(
					if self.selected_square == (x, y).into(){b"["}
					else if x > 0 && self.selected_square == (x-1, y).into(){b"]"}
					else if x > 0 && match self.get((x-1, y).into()){
						Square::Empty => false,
						Square::Line{to, from, ..} => to == &Some(Direction::East) || from  == &Direction::East,
						Square::Entry{dir, ..} => dir == &Some(Direction::East),
					} {
						if self.selected_flow == self.get((x-1, y).into()).value(){"\x1B[0;34m\u{2500}".as_bytes()}
						else{"\u{2500}".as_bytes()}
					} else {b" "}
				).map_err(|err| println!("{:?}", err)).ok();
				if self.selected_flow.is_some() && self.selected_flow == self.get((x, y).into()).value(){
					std::io::stdout().write_all(b"\x1B[0;34m").map_err(|err| println!("{:?}", err)).ok();
				}
				std::io::stdout().write_all(match self.get((x, y).into()){
					Square::Empty => " ".to_string(),
					Square::Line{to: Some(to), from, ..} => match (to, from){
						(Direction::North, Direction::East) |
						(Direction::East, Direction::North) => "\u{2514}",
						(Direction::North, Direction::South) |
						(Direction::South, Direction::North) => "\u{2502}",
						(Direction::North, Direction::West) |
						(Direction::West, Direction::North) => "\u{2518}",
						(Direction::East, Direction::South) |
						(Direction::South, Direction::East) => "\u{250c}",
						(Direction::East, Direction::West) |
						(Direction::West, Direction::East) => "\u{2500}",
						(Direction::South, Direction::West) |
						(Direction::West, Direction::South) => "\u{2510}",
						_ => panic!("invalid state")
					}.to_string(),
					Square::Entry{flow, ..} => String::from_utf8(vec![*flow as u8 + b'A']).unwrap(),
					Square::Line{to: None, from: dir, ..} => match dir{
						Direction::North => "\u{2575}",
						Direction::East => "\u{2576}",
						Direction::South => "\u{2577}",
						Direction::West => "\u{2574}",
					}.to_string(),
				}.as_bytes()).map_err(|err| println!("{:?}", err)).ok();
				std::io::stdout().write_all(b"\x1B[0;37m").map_err(|err| println!("{:?}", err)).ok();
			}
			std::io::stdout().write_all(
				if self.selected_square == (self.width as isize-1,y).into(){b"]"}else{b" "}
			).map_err(|err| println!("{:?}", err)).ok();
			std::io::stdout().write_all("\u{2502}\n".as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		}
		std::io::stdout().write_all("\u{2514}".as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		std::io::stdout().write_all("\u{2500}".repeat(self.width*2+1).as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		std::io::stdout().write_all("\u{2518}\n".as_bytes()).map_err(|err| println!("{:?}", err)).ok();

	}
	fn from_text(text: &[u8]) -> Result<Self, ()>{
		let mut width = 0;
		let mut height = 1;
		let mut board = Vec::new();
		for c in text{
			match c{
				b'.' => board.push(Square::Empty),
				b'\n' => {height += 1}
				c if *c >= b'A' && *c <= b'Z' => board.push(Square::Entry{ flow: (c - b'A') as u32, dir: None }),
				b'\r' => continue,
				_ => {return Err(())},
			}
			if height == 1{
				width += 1;
			}
		}
		if text.last() == Some(&('\n' as u8)){
			height -= 1;
		}
		return Ok(Self{
			width,
			height,
			board,
			selected_square: (0, 0).into(),
			selected_flow: None,
		})
	}

	fn clear_line(&mut self, mut dir: Option<Direction>, mut sqr: Coord) {
		while let Some(next) = dir{
			sqr += next.as_coord();
			match self.get_mut(sqr){
				Square::Line{to, ..} => {
					dir = *to;
					self.set(sqr, Square::Empty);
				}
				Square::Entry { dir: d, .. } => {
					*d = None;
					dir = None;
				}
				_=>unreachable!(),
			}
		}
	}
}


#[derive(PartialEq, Copy, Clone, Debug)]
enum Direction{
	North,
	East,
	South,
	West
}
impl Direction{
	fn as_coord(self) -> Coord{match self{
		Direction::North => (0, -1),
		Direction::East => (1, 0),
		Direction::South => (0, 1),
		Direction::West => (-1, 0),
	}.into()}
	fn oposite(self) -> Direction{match self{
		Direction::North => Self::South,
		Direction::East => Self::West,
		Direction::South => Self::North,
		Direction::West => Self::East,
	}}
}
#[derive(Debug)]
enum Square{
	Empty,
	Line{
		flow: u32,
		from: Direction,
		to: Option<Direction>
	},
	Entry{
		flow:u32,
		dir: Option<Direction>
	},
}
impl Square{
	fn value(&self) -> Option<u32>{match self{
		Self::Empty=>None,
		Self::Line{flow: i, ..} |
		Self::Entry{flow: i, ..}=>Some(i.clone())
	}}
	fn is_empty(&self) -> bool{match self{
		Self::Empty => true,
		_=>false,
	}}
	fn is_line(&self) -> bool{match self{
		Self::Line{..} => true,
		_=>false,
	}}
	fn is_entry(&self) -> bool{match self{
		Self::Entry{..} => true,
		_=>false,
	}}
}

fn move_line(board: &mut Board, dir: Direction) {
	if let Some(curr_flow) = board.selected_flow{
		match board.get_mut(board.selected_square+dir.as_coord()){
			Square::Empty => { // starts or continues a line
				board.set(board.selected_square+dir.as_coord(), Square::Line{
					flow: curr_flow,
					from: dir.oposite(),
					to: None,
				});
				match board.get_mut(board.selected_square){
					Square::Entry { dir: to, .. } | Square::Line { to, .. } => *to = Some(dir),
					Square::Empty => unreachable!(),
				};
				board.selected_square += dir.as_coord();
			}
			Square::Entry { flow, dir: to } | Square::Line { flow, to, .. }
			if *flow == curr_flow && *to == Some(dir.oposite()) => { // undo a line
				*to = None;
				assert!(board.get(board.selected_square).is_line());
				board.set(board.selected_square, Square::Empty);
				board.selected_square += dir.as_coord();
			},
			Square::Entry { flow, dir: exit } if *flow == curr_flow && *exit == None => { // end a line
				*exit = Some(dir.oposite());
				match board.get_mut(board.selected_square){
					Square::Entry { dir: to, .. } | Square::Line { to, .. } => *to = Some(dir),
					Square::Empty => unreachable!(),
				};
				board.selected_flow = None;
				board.selected_square += dir.as_coord();
			},
			_ => {}, // blocked
		}
	}else{ // here we are just moving the cursor
		board.selected_square += dir.as_coord();
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let mut buf = vec!();
	let mut board = Board::from_text(if args.len() == 2{
		let mut input = File::open(&args[1]).unwrap();
		let len = input.seek(SeekFrom::End(0)).unwrap();
		buf.resize(len as usize-1, 0);
		input.seek(SeekFrom::Start(0)).unwrap();
		input.read_exact(&mut buf).unwrap();
		&buf
	}else{b".....E\n......\n.DA...\n...B.D\n.B.EAC\n...C.."}).unwrap();
	// board.board.push(Square::Line { flow: 0, from: Direction::East, to: Some(Direction::South) });
	// let len = board.board.len();
	// board.board.swap(1, len-1);
	// board.board.pop();
	let getch = Getch::new();
	std::io::stdout().write_all("\n".repeat(board.height+2).as_bytes()).map_err(|err| println!("{:?}", err)).ok();
	let mut playing = true;
	'main : loop {
		std::io::stdout().write_all(("\x1B[".to_owned()+&(board.height+3).to_string()+"A\n").as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		board.display();
		if !playing{break;}
		match getch.getch() {
			Ok(3) => return,
			Ok(77)|Ok(100) if board.selected_square.x < board.width as isize - 1 => move_line(&mut board, Direction::East),
			Ok(75)|Ok(97) if board.selected_square.x > 0 => move_line(&mut board, Direction::West),
			Ok(80)|Ok(115) if board.selected_square.y < board.height as isize - 1 => move_line(&mut board, Direction::South),
			Ok(72)|Ok(119) if board.selected_square.y > 0 => move_line(&mut board, Direction::North),
			Ok(32)|Ok(13) => board.selected_flow = if board.selected_flow == None {
				match board.get_mut(board.selected_square){
					Square::Entry { dir, .. } => {
						let mut dir = dir.take();
						let sqr = if let Some(dir) = dir{
							match board.get(board.selected_square+dir.as_coord()){Square::Line{ from, .. } => {
								if dir.oposite() == *from {board.selected_square}
								else{board.get_other_entry(board.selected_square).unwrap()}
							},_ => unreachable!(),}
						}else{board.get_other_entry(board.selected_square).unwrap()};
						if sqr != board.selected_square{
							if let Square::Entry { dir: new_dir, .. } = board.get_mut(sqr){
								dir = new_dir.take();
							}else{unreachable!();}
						}
						board.clear_line(dir, sqr);
					},
					Square::Line { to: dir, .. } => {
						let dir = dir.take();
						board.clear_line(dir, board.selected_square);
					},
					_ => {}
				}
				board.get(board.selected_square).value()
			}else{None},
			Ok(_)|Err(_) => continue,
		};
		for i in 0..board.width * board.height{
			match board.board[i] {
				Square::Entry { dir: None, .. } | Square::Empty => {continue 'main;},
				_ => {}
			}
		}
		playing = false;
	}
	println!("YOU WON!!");
}
