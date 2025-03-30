use bitset_core::BitSet;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Addr {
    x: usize,
    y: usize,
    b: usize,
}
impl Addr {
    pub fn new(x: usize, y: usize) -> Addr {
        Addr {
            x,
            y,
            b: (y / 3) * 3 + (x / 3),
        }
    }
}
#[derive(Default, PartialEq, Eq, Debug, Clone)]
enum Field {
    #[default]
    Empty,
    Set(u8),
}
impl Field {}

#[derive(Clone)]
struct Board {
    // open: BTreeSet<Addr>,
    open: Vec<Addr>,
    fields: [[Field; 9]; 9],
    h_free: [u16; 9],
    v_free: [u16; 9],
    b_free: [u16; 9],
}
impl Default for Board {
    fn default() -> Self {
        Self {
            open: (0..9)
                .flat_map(|y| (0..9).map(move |x| Addr::new(x, y)))
                .collect(),
            fields: Default::default(),
            h_free: [0b111111111; 9],
            v_free: [0b111111111; 9],
            b_free: [0b111111111; 9],
        }
    }
}
#[derive(Clone, Debug)]
struct Edit {
    addr: Addr,
    num: u8,
}
impl Board {
    pub fn from_line(line: &str) -> Board {
        let mut board = Board::default();
        let mut line = line.chars();
        for i in 0..9 {
            for j in 0..9 {
                let c = line.next().expect("line ended early");

                if c.is_numeric() {
                    let num = c.to_digit(10).unwrap() as usize;
                    if num < 1 {
                        panic!("bad num in input: {}", num);
                    }
                    let num = num - 1;

                    let addr = Addr::new(j, i);
                    for i in 0..board.open.len() {
                        if board.open[i] == addr {
                            board.open.remove(i);
                            break;
                        }
                    }

                    board.manipulate(addr, num);
                }
            }
        }
        board
    }

    const USE_UNSAFE: bool = false;
    fn get_h_mut(&mut self, addr: Addr) -> &mut u16 {
        if !Self::USE_UNSAFE {
            &mut self.h_free[addr.y]
        } else {
            unsafe { self.h_free.get_unchecked_mut(addr.y) }
        }
    }
    fn get_v_mut(&mut self, addr: Addr) -> &mut u16 {
        if !Self::USE_UNSAFE {
            &mut self.v_free[addr.x]
        } else {
            unsafe { self.v_free.get_unchecked_mut(addr.x) }
        }
    }
    fn get_b_mut(&mut self, addr: Addr) -> &mut u16 {
        if !Self::USE_UNSAFE {
            &mut self.b_free[addr.b]
        } else {
            unsafe { self.b_free.get_unchecked_mut(addr.b) }
        }
    }
    fn get_h(&self, addr: Addr) -> u16 {
        if !Self::USE_UNSAFE {
            self.h_free[addr.y]
        } else {
            unsafe { *self.h_free.get_unchecked(addr.y) }
        }
    }
    fn get_v(&self, addr: Addr) -> u16 {
        if !Self::USE_UNSAFE {
            self.v_free[addr.x]
        } else {
            unsafe { *self.v_free.get_unchecked(addr.x) }
        }
    }
    fn get_b(&self, addr: Addr) -> u16 {
        if !Self::USE_UNSAFE {
            self.b_free[addr.b]
        } else {
            unsafe { *self.b_free.get_unchecked(addr.b) }
        }
    }
    pub fn manipulate(&mut self, addr: Addr, num: usize) -> Edit {
        assert!(num < 9);

        self.get_h_mut(addr).bit_reset(num);
        self.get_v_mut(addr).bit_reset(num);
        self.get_b_mut(addr).bit_reset(num);

        let f = &mut self.fields[addr.y][addr.x];
        assert_eq!(*f, Field::Empty);
        *f = Field::Set(num as u8);
        Edit {
            addr,
            num: num as u8,
        }
    }
    pub fn rollback(&mut self, edit: Edit) {
        self.get_h_mut(edit.addr).bit_set(edit.num as usize);
        self.get_v_mut(edit.addr).bit_set(edit.num as usize);
        self.get_b_mut(edit.addr).bit_set(edit.num as usize);
        let f = &mut self.fields[edit.addr.y][edit.addr.x];
        assert_eq!(*f, Field::Set(edit.num));
        *f = Field::Empty;
    }
    pub fn candidates_for(&self, addr: Addr) -> u16 {
        self.get_h(addr) & self.get_v(addr) & self.get_b(addr)
    }
    pub fn solve(&mut self) -> Option<Vec<Vec<Edit>>> {
        if self.open.is_empty() {
            return Some(vec![Vec::new()]);
            // return Some(Vec::new());
        }
        let mut edit_alternatives: Vec<Vec<Edit>> = Default::default();
        let mut min_candidates = 0u16;
        let mut min_i = usize::MAX;
        {
            let mut min = u32::MAX;
            for (i, field) in self.open.iter().enumerate() {
                let candidates = self.candidates_for(*field);
                let num = candidates.count_ones();
                if num < min {
                    min_i = i;
                    min = num;
                    min_candidates = candidates;
                }
                if min == 1 {
                    break;
                }
            }
            if min_i == usize::MAX {
                // return Some(vec![Vec::new()]);
                panic!("no minimal candidate found. should be impossible.")
            }
        }
        let Addr { x, y, b: _ } = self.open[min_i];
        self.open.swap_remove(min_i);
        let open = self.open.len();
        for c in 0..9 {
            if !min_candidates.bit_test(c) {
                continue;
            }
            let edit = self.manipulate(Addr::new(x, y), c.into());
            match self.solve() {
                Some(solution_alternatives) => {
                    self.rollback(edit.clone());
                    let lens = solution_alternatives
                        .iter()
                        .map(|alt| alt.len())
                        .collect::<Vec<_>>();
                    // println!("lens: {} {:?}", open, lens);
                    for mut solution_edits in solution_alternatives {
                        solution_edits.push(edit.clone());
                        edit_alternatives.push(solution_edits);
                    }
                }
                None => {
                    self.rollback(edit.clone());
                }
            }
        }
        self.open.push(Addr::new(x, y));
        if edit_alternatives.is_empty() {
            None
        } else {
            Some(edit_alternatives)
        }
    }
    pub fn print(&self) {
        for y in 0..9 {
            for x in 0..9 {
                match self.fields[y][x] {
                    Field::Empty => print!(". "),
                    Field::Set(num) => print!("{} ", num + 1),
                }
            }
            println!();
        }
    }
}

fn main() {
    let args = args();
    if args.len() != 2 {
        println!("missing filename");
        return;
    }

    let filename = args.last().unwrap();

    println!("{filename}");
    let file = File::open(filename).unwrap();
    for line in io::BufReader::new(file).lines() {
        let line = line.unwrap();
        let mut board = Board::from_line(&line[..]);
        println!("=========================\nsolving:\n");
        board.print();
        let solved = board.solve();
        match solved {
            Some(alternative_edits) => {
                for mut edits in alternative_edits {
                    println!("solved:");
                    // println!("edits: {} {:?}", edits.len(), edits);
                    let mut board = board.clone();
                    while let Some(edit) = edits.pop() {
                        board.manipulate(edit.addr, edit.num.into());
                    }
                    board.print();
                }
            }
            None => println!("unsolvable"),
        }
    }
    println!("end");
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_manipulate() {
        let mut board = Board::default();
        let a = Addr::new(0, 0);
        assert!(board.get_h(a).count_ones() == 9);
        let edit = board.manipulate(a, 0);
        assert!(board.get_h(a).count_ones() == 8);
        assert!(board.get_h(a).bit_test(0) == false);
        board.rollback(edit);
        assert!(board.get_h(a).count_ones() == 9);

        let a = Addr::new(8, 8);
        let edit = board.manipulate(a, 6);
        assert!(board.get_h(a).count_ones() == 8);
        assert!(board.get_h(a).bit_test(6) == false);
        assert!(board.get_v(a).count_ones() == 8);
        assert!(board.get_v(a).bit_test(6) == false);
        assert!(board.get_b(a).count_ones() == 8);
        assert!(board.get_b(a).bit_test(6) == false);
        board.rollback(edit);
        assert!(board.get_h(a).count_ones() == 9);
    }
    #[test]
    pub fn test_board_init() {
        let mut board = Board::default();
        assert_eq!(board.open.len(), 9 * 9);
        assert!(board.open.contains(&Addr::new(0, 0)));
        assert!(board.open.contains(&Addr::new(8, 8)));
        assert!(board.open.contains(&Addr::new(0, 8)));
        assert!(board.open.contains(&Addr::new(8, 0)));
        assert!(board.open.contains(&Addr::new(3, 7)));
        // let edit = board.manipulate(Addr::new(3, 7), 7);
        // assert!(!board.open.contains(&Addr::new(3, 7)));

        // assert_eq!(board.open.len(), 9 * 9 - 1);
        // board.rollback(edit);
        // assert!(board.open.contains(&Addr::new(3, 7)));
        // assert_eq!(board.open.len(), 9 * 9);
    }
}
