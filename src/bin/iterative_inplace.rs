use bitset_core::BitSet;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
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
#[derive(Clone, Debug, Default)]
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

                    board.manipulate(&addr, num);
                }
            }
        }
        board
    }

    fn get_h_mut(&mut self, addr: &Addr) -> &mut u16 {
        &mut self.h_free[addr.y]
    }
    fn get_v_mut(&mut self, addr: &Addr) -> &mut u16 {
        &mut self.v_free[addr.x]
    }
    fn get_b_mut(&mut self, addr: &Addr) -> &mut u16 {
        &mut self.b_free[addr.b]
    }
    fn get_h(&self, addr: &Addr) -> u16 {
        self.h_free[addr.y]
    }
    fn get_v(&self, addr: &Addr) -> u16 {
        self.v_free[addr.x]
    }
    fn get_b(&self, addr: &Addr) -> u16 {
        self.b_free[addr.b]
    }
    pub fn manipulate(&mut self, addr: &Addr, num: usize) -> Edit {
        assert!(num < 9);

        self.get_h_mut(addr).bit_reset(num);
        self.get_v_mut(addr).bit_reset(num);
        self.get_b_mut(addr).bit_reset(num);

        let f = &mut self.fields[addr.y][addr.x];
        assert_eq!(*f, Field::Empty);
        *f = Field::Set(num as u8);
        Edit {
            addr: *addr,
            num: num as u8,
        }
    }
    pub fn rollback(&mut self, edit: Edit) {
        self.get_h_mut(&edit.addr).bit_set(edit.num as usize);
        self.get_v_mut(&edit.addr).bit_set(edit.num as usize);
        self.get_b_mut(&edit.addr).bit_set(edit.num as usize);
        let f = &mut self.fields[edit.addr.y][edit.addr.x];
        assert_eq!(*f, Field::Set(edit.num));
        *f = Field::Empty;
    }
    pub fn candidates_for(&self, addr: &Addr) -> u16 {
        self.get_h(addr) & self.get_v(addr) & self.get_b(addr)
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

#[derive(Debug, Default)]
struct IterState2 {
    candidates: u16,
    edit: Edit,
    addr: Addr,
}
#[derive(Debug)]
enum IterState {
    Initial,
    Applied {
        candidates: u16,
        edit: Edit,
        addr: Addr,
    },
}
fn solve(board: &mut Board) -> bool {
    let mut stack = Vec::<IterState2>::new();

    stack.push(IterState2::default());
    let mut max_depth: usize = 0;
    let mut num_steps: usize = 0;

    loop {
        max_depth = max_depth.max(stack.len());
        num_steps += 1;
        let cur_state = stack.last_mut().expect("stack underflow");
        if cur_state.candidates == 0 {
            if board.open.is_empty() {
                board.print();
                println!("max depth: {}, steps: {}", max_depth, num_steps);
                return true;
            }
            let (mut min_candidates, min_i) = best_candidate(board);
            let addr = board.open.swap_remove(min_i);

            println!("best candidate: {:?} {}", addr, min_candidates);
            let test = min_candidates.trailing_zeros();
            if test >= 9 {
                // unsolvable -> return / backtrack
                stack.pop();
                board.open.push(addr);
            } else {
                // test candidate field:
                // 1. knock out lowest bit
                // 2. 'recursion'
                min_candidates.bit_reset(test as usize);
                let edit = board.manipulate(&addr, test as usize);
                // stack.push(IterState::Applied {
                //     candidates: min_candidates,
                //     edit,
                //     addr,
                // });
                cur_state.candidates = min_candidates;
                cur_state.edit = edit;
                cur_state.addr = addr;
                stack.push(IterState2::default());
            }
        } else {
            board.rollback(cur_state.edit.clone());
            let test = cur_state.candidates.trailing_zeros();
            // println!("test: {} {}", test, candidates);
            if test < 9 {
                // test candidate field:
                // 1. knock out lowest bit
                // 2. 'recursion'
                cur_state.candidates.bit_reset(test as usize);
                let edit = board.manipulate(&cur_state.addr, test as usize);
                cur_state.edit = edit;

                // stack.push(IterState::Applied {
                //     candidates,
                //     edit,
                //     addr,
                // });
                stack.push(IterState2::default());
            } else {
                // all candidate numbers knocked out but not solved -> return / backtrack
                board.open.push(cur_state.addr);
                stack.pop();
            }
        }
    }
}

fn best_candidate(board: &mut Board) -> (u16, usize) {
    let mut min_candidates = 0u16;
    let mut min_i = usize::MAX;
    {
        let mut min = u32::MAX;
        for (i, field) in board.open.iter().enumerate() {
            let candidates = board.candidates_for(field);
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
            panic!("no minimal candidate found. should be impossible.")
        }
    }
    (min_candidates, min_i)
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
        let solved = solve(&mut board);
        match solved {
            true => {
                println!("solved.");
            }
            false => println!("unsolvable"),
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
        let a = &Addr::new(0, 0);
        assert!(board.get_h(a).count_ones() == 9);
        let edit = board.manipulate(a, 0);
        assert!(board.get_h(a).count_ones() == 8);
        assert!(board.get_h(a).bit_test(0) == false);
        board.rollback(edit);
        assert!(board.get_h(a).count_ones() == 9);

        let a = &Addr::new(8, 8);
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
