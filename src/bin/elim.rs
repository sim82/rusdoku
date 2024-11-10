use bit_set::BitSet;
use bit_vec::BitVec;
use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Addr {
    x: usize,
    y: usize,
}
impl Addr {
    pub fn new(x: usize, y: usize) -> Addr {
        Addr { x, y }
    }
}
#[derive(Default, PartialEq, Eq, Debug)]
enum Field {
    #[default]
    Empty,
    Set(u8),
}
impl Field {}
struct Board {
    // open: BTreeSet<Addr>,
    open: Vec<Addr>,
    fields: [[Field; 9]; 9],
    h_free: [BitVec; 9],
    v_free: [BitVec; 9],
    b_free: [BitVec; 9],
}
impl Default for Board {
    fn default() -> Self {
        Self {
            open: (0..9)
                .flat_map(|y| (0..9).map(move |x| Addr::new(x, y)))
                .collect(),
            fields: Default::default(),
            h_free: std::array::from_fn(|_| BitVec::from_elem(9, true)),
            v_free: std::array::from_fn(|_| BitVec::from_elem(9, true)),
            b_free: std::array::from_fn(|_| BitVec::from_elem(9, true)),
        }
    }
}
#[derive(Clone)]
struct Edit {
    addr: Addr,
    num: u8,
}
impl Edit {}
impl Board {
    pub fn from_line(line: &str) -> Board {
        let mut board = Board::default(); //let mut line = line;
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

    fn get_h_mut(&mut self, addr: Addr) -> &mut BitVec {
        &mut self.h_free[addr.y]
    }
    fn get_v_mut(&mut self, addr: Addr) -> &mut BitVec {
        &mut self.v_free[addr.x]
    }
    fn get_b_mut(&mut self, addr: Addr) -> &mut BitVec {
        &mut self.b_free[(addr.y / 3) * 3 + (addr.x / 3)]
    }
    fn get_h(&self, addr: Addr) -> &BitVec {
        &self.h_free[addr.y]
    }
    fn get_v(&self, addr: Addr) -> &BitVec {
        &self.v_free[addr.x]
    }
    fn get_b(&self, addr: Addr) -> &BitVec {
        &self.b_free[(addr.y / 3) * 3 + (addr.x / 3)]
    }
    pub fn manipulate(&mut self, addr: Addr, num: usize) -> Edit {
        assert!(num < 9);

        self.get_h_mut(addr).set(num, false);
        self.get_v_mut(addr).set(num, false);
        self.get_b_mut(addr).set(num, false);

        let edit = Edit {
            addr,
            num: num as u8,
        };

        let f = &mut self.fields[addr.y][addr.x];
        assert_eq!(*f, Field::Empty);
        *f = Field::Set(num as u8);
        // let res = self.open.remove(&addr);
        // assert!(res);
        edit
    }
    pub fn rollback(&mut self, edit: Edit) {
        self.get_h_mut(edit.addr).set(edit.num as usize, true);
        self.get_v_mut(edit.addr).set(edit.num as usize, true);
        self.get_b_mut(edit.addr).set(edit.num as usize, true);
        let f = &mut self.fields[edit.addr.y][edit.addr.x];
        assert_eq!(*f, Field::Set(edit.num));
        *f = Field::Empty;
        // self.open.insert(edit.addr);
    }
    pub fn candidates_for(&self, addr: Addr) -> BitVec {
        let mut bs = self.get_h(addr).clone();
        bs.and(self.get_v(addr));
        bs.and(self.get_b(addr));

        bs
    }
    pub fn solve(&mut self) -> Option<Vec<Edit>> {
        let mut min_i = usize::MAX;
        let mut min = u64::MAX;
        for (i, field) in self.open.iter().enumerate() {
            let num = self.candidates_for(*field).count_ones();
            if num < min {
                min_i = i;
                min = num;
            }
            if min == 1 {
                break;
            }
        }
        if min_i == usize::MAX {
            return Some(Vec::new());
        }
        let Addr { x, y } = self.open[min_i];
        self.open.swap_remove(min_i);
        // println!("try: {x} {y}");
        let candidates = self.candidates_for(Addr::new(x, y));
        // for c in candidates {
        for c in 0..9 {
            if candidates.get(c) != Some(true) {
                continue;
            }
            // println!("c: {}", c);
            let edit = self.manipulate(Addr::new(x, y), c.into());
            match self.solve() {
                Some(mut edits) => {
                    self.rollback(edit.clone());
                    edits.push(edit);
                    return Some(edits);
                }
                None => {
                    self.rollback(edit.clone());
                }
            }
            // if self.solve() {
            // return true;
            // } else {
            // self.rollback(edit);
            // }
        }
        self.open.push(Addr::new(x, y));
        None
    }
    // self.print();
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
    // let mut h_list = ListH::default();
    // let mut v_list = ListV::default();
    // let mut b_list = ListB::default();

    // let mut f = Field::new_rc(0, 0);
    // h_list.cursor_mut().insert_after(f.clone());
    // v_list.cursor_mut().insert_after(f.clone());
    // b_list.cursor_mut().insert_after(f.clone());

    let file = File::open("top95.txt").unwrap();
    for line in io::BufReader::new(file).lines() {
        let line = line.unwrap();
        let mut board = Board::from_line(&line[..]);
        let solved = board.solve();
        match solved {
            Some(mut edits) => {
                while let Some(edit) = edits.pop() {
                    board.manipulate(edit.addr, edit.num.into());
                }
                println!("solved:");
                board.print();
            }
            None => println!("unsolvable"),
        }
        // println!("solved: {}", solved);
        // let line = line.unwrap();
        // board.borrow_mut().init(&line[..]);
        // println!("{:?}", board.borrow().v_free[0]);
    }
    println!("end");
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_manipulate() {
        let mut board = Board::default();
        let a = Addr::new(0, 0_);
        assert!(board.get_h(a).count_ones() == 9);
        let edit = board.manipulate(Addr::new(0, 0), 0);
        assert!(board.get_h(a).count_ones() == 8 && board.get_h(a).get(0) == Some(false));
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
