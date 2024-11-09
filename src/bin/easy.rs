use intrusive_collections::intrusive_adapter;
use intrusive_collections::{LinkedList, LinkedListLink};
use std::cell::RefCell;
use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, BufRead, Empty};
use std::iter::FromIterator;
use std::rc::Rc;
use typenum::consts::U9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Addr {
    x: usize,
    y: usize,
}
impl Addr {
    pub fn new(x: usize, y: usize) -> Addr {
        Addr { x, y }
    }
    pub fn get_h(&self) -> Vec<Addr> {
        (0..self.x)
            .into_iter()
            .chain((self.x + 1)..9)
            .map(|x| Addr::new(x, self.y))
            .collect()
    }
    pub fn get_v(&self) -> Vec<Addr> {
        (0..self.y)
            .into_iter()
            .chain((self.y + 1)..9)
            .map(|y| Addr::new(self.x, y))
            .collect()
    }
    pub fn get_b(&self) -> Vec<Addr> {
        let xb = (self.x / 3) * 3;
        let yb = (self.y / 3) * 3;
        [
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 1),
            (1, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ]
        .iter()
        .filter_map(|(x, y)| {
            if !(xb + x == self.x && yb + y == self.y) {
                Some(Addr::new(xb + x, yb + y))
            } else {
                None
            }
        })
        .collect()
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
    fields: [[Field; 9]; 9],
}

#[derive(Clone)]
struct Edit {
    addr: Addr,
    num: u8,
}
impl Edit {}
impl Board {
    pub fn from_line(line: &str) -> Board {
        let mut board = Board {
            fields: Default::default(),
        };
        //let mut line = line;
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

                    board.manipulate(Addr::new(j, i), num);
                }
            }
        }
        board
    }

    pub fn manipulate(&mut self, addr: Addr, num: usize) -> Edit {
        let edit = Edit {
            addr,
            num: num as u8,
        };
        self.apply(&edit);
        edit
    }
    pub fn apply(&mut self, edit: &Edit) {
        let f = &mut self.fields[edit.addr.y][edit.addr.x];
        assert!(*f == Field::Empty);
        assert!(edit.num < 9);
        *f = Field::Set(edit.num as u8);
    }
    pub fn rollback(&mut self, edit: Edit) {
        let f = &mut self.fields[edit.addr.y][edit.addr.x];
        assert_eq!(*f, Field::Set(edit.num));
        *f = Field::Empty
    }
    pub fn candidates_for(&self, addr: Addr) -> HashSet<u8> {
        if self.fields[addr.y][addr.x] != Field::Empty {
            return Default::default();
        }
        // let v = addr
        //     .get_v()
        //     .into_iter()
        //     .filter_map(|Addr { x, y }| match self.fields[y][x] {
        //         Field::Empty => None,
        //         Field::Set(num) => Some(num),
        //     });
        // let b = addr
        //     .get_b()
        //     .into_iter()
        //     .filter_map(|Addr { x, y }| match self.fields[y][x] {
        //         Field::Empty => None,
        //         Field::Set(num) => Some(num),
        //     });
        // h.chain(v).chain(b).collect::<HashSet<_>>().into()
        let r: HashSet<u8> = [0, 1, 2, 3, 4, 5, 6, 7, 8].into();
        let p: HashSet<u8> = addr
            .get_h()
            .into_iter()
            .chain(addr.get_v().into_iter())
            .chain(addr.get_b().into_iter())
            .filter_map(|Addr { x, y }| match self.fields[y][x] {
                Field::Empty => None,
                Field::Set(num) => Some(num),
            })
            .collect::<HashSet<_>>()
            .into();

        r.difference(&p).cloned().collect()
    }
    pub fn solve(&mut self) -> Option<Vec<Edit>> {
        for y in 0..8 {
            for x in 0..8 {
                if self.fields[y][x] != Field::Empty {
                    continue;
                }
                // println!("try: {x} {y}");
                let candidates = self.candidates_for(Addr::new(x, y));
                for c in candidates {
                    let edit = self.manipulate(Addr::new(x, y), c as usize);
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
                return None;
            }
        }
        // self.print();
        return Some(Vec::new());
    }
    pub fn print(&self) {
        for y in 0..8 {
            for x in 0..8 {
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
                    board.apply(&edit)
                }
                println!("solved: ");
                board.print();
            }
            None => println!("unsolvable"),
        }
        // println!("solved: {}", solved);
        // let line = line.unwrap();
        // board.borrow_mut().init(&line[..]);
        // println!("{:?}", board.borrow().v_free[0]);
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn addr_h() {
        assert_eq!(
            &Addr::new(0, 2).get_h()[..],
            &[
                Addr::new(1, 2),
                Addr::new(2, 2),
                Addr::new(3, 2),
                Addr::new(4, 2),
                Addr::new(5, 2),
                Addr::new(6, 2),
                Addr::new(7, 2),
                Addr::new(8, 2)
            ]
        );
        assert_eq!(
            &Addr::new(4, 7).get_h()[..],
            &[
                Addr::new(0, 7),
                Addr::new(1, 7),
                Addr::new(2, 7),
                Addr::new(3, 7),
                Addr::new(5, 7),
                Addr::new(6, 7),
                Addr::new(7, 7),
                Addr::new(8, 7)
            ]
        );
        assert_eq!(
            &Addr::new(8, 0).get_h()[..],
            &[
                Addr::new(0, 0),
                Addr::new(1, 0),
                Addr::new(2, 0),
                Addr::new(3, 0),
                Addr::new(4, 0),
                Addr::new(5, 0),
                Addr::new(6, 0),
                Addr::new(7, 0),
            ]
        );
        // println!("h: {:?}", addr.get_h());
    }
    #[test]
    pub fn addr_v() {
        assert_eq!(
            &Addr::new(2, 0).get_v()[..],
            &[
                Addr::new(2, 1),
                Addr::new(2, 2),
                Addr::new(2, 3),
                Addr::new(2, 4),
                Addr::new(2, 5),
                Addr::new(2, 6),
                Addr::new(2, 7),
                Addr::new(2, 8)
            ]
        );
        assert_eq!(
            &Addr::new(7, 4).get_v()[..],
            &[
                Addr::new(7, 0),
                Addr::new(7, 1),
                Addr::new(7, 2),
                Addr::new(7, 3),
                Addr::new(7, 5),
                Addr::new(7, 6),
                Addr::new(7, 7),
                Addr::new(7, 8)
            ]
        );
        assert_eq!(
            &Addr::new(0, 8).get_v()[..],
            &[
                Addr::new(0, 0),
                Addr::new(0, 1),
                Addr::new(0, 2),
                Addr::new(0, 3),
                Addr::new(0, 4),
                Addr::new(0, 5),
                Addr::new(0, 6),
                Addr::new(0, 7),
            ]
        );
        // println!("h: {:?}", addr.get_h());
    }
    #[test]
    pub fn addr_b() {
        assert_eq!(
            &Addr::new(0, 0).get_b()[..],
            &[
                // Addr::new(0, 0),
                Addr::new(1, 0),
                Addr::new(2, 0),
                Addr::new(0, 1),
                Addr::new(1, 1),
                Addr::new(2, 1),
                Addr::new(0, 2),
                Addr::new(1, 2),
                Addr::new(2, 2),
            ]
        );
        assert_eq!(
            &Addr::new(5, 4).get_b()[..],
            &[
                Addr::new(3, 3),
                Addr::new(4, 3),
                Addr::new(5, 3),
                Addr::new(3, 4),
                Addr::new(4, 4),
                // Addr::new(5, 4),
                Addr::new(3, 5),
                Addr::new(4, 5),
                Addr::new(5, 5)
            ]
        );
        assert_eq!(
            &Addr::new(8, 8).get_b()[..],
            &[
                Addr::new(6, 6),
                Addr::new(7, 6),
                Addr::new(8, 6),
                Addr::new(6, 7),
                Addr::new(7, 7),
                Addr::new(8, 7),
                Addr::new(6, 8),
                Addr::new(7, 8),
                // Addr::new(8, 8),
            ]
        );
        // println!("h: {:?}", addr.get_h());
    }
    #[test]
    pub fn get_chandidates() {
        let mut board = Board {
            fields: Default::default(),
        };

        board.manipulate(Addr::new(0, 0), 0);
        board.manipulate(Addr::new(1, 0), 1);
        board.manipulate(Addr::new(2, 0), 2);
        board.manipulate(Addr::new(3, 0), 3);
        board.manipulate(Addr::new(4, 0), 4);
        board.manipulate(Addr::new(5, 0), 5);
        board.manipulate(Addr::new(6, 0), 6);
        board.manipulate(Addr::new(7, 0), 7);
        assert_eq!(board.candidates_for(Addr::new(8, 0)), [8].into());
        assert_eq!(
            board.candidates_for(Addr::new(8, 1)),
            [0, 1, 2, 3, 4, 5, 8].into()
        );
        assert_eq!(
            board.candidates_for(Addr::new(0, 8)),
            [1, 2, 3, 4, 5, 6, 7, 8].into()
        );
    }
}
