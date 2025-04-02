use bitset_core::BitSet;
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead};

#[rustfmt::skip]
const F2H : [usize; 9 * 9] = [
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
    0,1,2,3,4,5,6,7,8, 
];
#[rustfmt::skip]
const F2V : [usize; 9 * 9] = [
    0,0,0,0,0,0,0,0,0,
    1,1,1,1,1,1,1,1,1,
    2,2,2,2,2,2,2,2,2,
    3,3,3,3,3,3,3,3,3,
    4,4,4,4,4,4,4,4,4,
    5,5,5,5,5,5,5,5,5,
    6,6,6,6,6,6,6,6,6,
    7,7,7,7,7,7,7,7,7,
    8,8,8,8,8,8,8,8,8,
    
];
#[rustfmt::skip]
const F2B : [usize; 9 * 9] = [
    0,0,0,1,1,1,2,2,2,
    0,0,0,1,1,1,2,2,2,
    0,0,0,1,1,1,2,2,2,
    3,3,3,4,4,4,5,5,5,
    3,3,3,4,4,4,5,5,5,
    3,3,3,4,4,4,5,5,5,
    6,6,6,7,7,7,8,8,8,
    6,6,6,7,7,7,8,8,8,
    6,6,6,7,7,7,8,8,8,
];

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
enum Field {
    #[default]
    Empty,
    Set(u8),
}
impl Field {}

#[derive(Clone)]
struct Board {
    open: Vec<usize>,
    fields: [Field; 9 * 9],
    h_free: [u16; 9],
    v_free: [u16; 9],
    b_free: [u16; 9],
}
impl Default for Board {
    fn default() -> Self {
        Self {
            open: (0..9)
                .flat_map(|y| (0..9).map(move |x| y * 9 + x))
                .collect(),
            fields: [Field::default(); 9 * 9],
            h_free: [0b111111111; 9],
            v_free: [0b111111111; 9],
            b_free: [0b111111111; 9],
        }
    }
}
#[derive(Clone, Debug, Default)]
struct Edit {
    field: usize,
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

                    // let addr = Addr::new(j, i);
                    let field = i * 9 + j;
                    for i in 0..board.open.len() {
                        if board.open[i] == field {
                            board.open.remove(i);
                            break;
                        }
                    }

                    board.manipulate(field, num);
                }
            }
        }
        board
    }

    pub fn manipulate(&mut self, field: usize, num: usize) -> Edit {
        assert!(num < 9);

        self.h_free[F2H[field]].bit_reset(num);
        self.v_free[F2V[field]].bit_reset(num);
        self.b_free[F2B[field]].bit_reset(num);

        // let f = &mut self.fields[addr.y][addr.x];
        let f = &mut self.fields[field];
        assert_eq!(*f, Field::Empty);
        *f = Field::Set(num as u8);
        Edit {
            field,
            num: num as u8,
        }
    }
    pub fn rollback(&mut self, edit: Edit) {
        self.h_free[F2H[edit.field]].bit_set(edit.num as usize);
        self.v_free[F2V[edit.field]].bit_set(edit.num as usize);
        self.b_free[F2B[edit.field]].bit_set(edit.num as usize);
        let f = &mut self.fields[edit.field];
        assert_eq!(*f, Field::Set(edit.num));
        *f = Field::Empty;
    }
    pub fn candidates_for(&self, field: usize) -> u16 {
        self.h_free[F2H[field]] & self.v_free[F2V[field]] & self.b_free[F2B[field]]
    }
    pub fn print(&self) {
        for y in 0..9 {
            for x in 0..9 {
                match self.fields[y * 9 + x] {
                    Field::Empty => print!(". "),
                    Field::Set(num) => print!("{} ", num + 1),
                }
            }
            println!();
        }
    }
}

fn solve(board: &mut Board) -> bool {
    let mut candidates_stack = Vec::<u16>::new();
    let mut edit_stack = Vec::<Edit>::new();
    let mut field_stack = Vec::<usize>::new();
    // let mut addr_stack = Ve

    // let mut stack = Vec::<IterState2>::new();

    // stack.push(IterState2::default());
    candidates_stack.push(u16::MAX);
    edit_stack.push(Edit::default());
    field_stack.push(usize::MAX);

    let mut max_depth: usize = 0;
    let mut num_steps: usize = 0;

    loop {
        max_depth = max_depth.max(candidates_stack.len());
        num_steps += 1;
        // let cur_state = stack.last_mut().expect("stack underflow");
        let cur_candidates = candidates_stack.last_mut().expect("candidate stack empty");
        let cur_edit = edit_stack.last_mut().expect("edit stack empty");
        let cur_field = field_stack.last_mut().expect("field stack empty");

        if *cur_candidates == u16::MAX {
            if board.open.is_empty() {
                board.print();
                println!("max depth: {}, steps: {}", max_depth, num_steps);
                return true;
            }
            let (mut min_candidates, min_i) = best_candidate(board);
            let field = board.open.swap_remove(min_i);

            // println!("best candidate: {:?} {}", addr, min_candidates);
            let test = min_candidates.trailing_zeros();
            if test >= 9 {
                // unsolvable -> return / backtrack
                // stack.pop();
                candidates_stack.pop();
                edit_stack.pop();
                field_stack.pop();
                board.open.push(field);
            } else {
                // test candidate field:
                // 1. knock out lowest bit
                // 2. 'recursion'
                min_candidates.bit_reset(test as usize);
                let edit = board.manipulate(field, test as usize);
                *cur_candidates = min_candidates;
                *cur_edit = edit;
                *cur_field = field;
                // stack.push(IterState2::default());
                candidates_stack.push(u16::MAX);
                edit_stack.push(Edit::default());
                field_stack.push(usize::MAX);
            }
        } else {
            board.rollback(cur_edit.clone());
            let test = cur_candidates.trailing_zeros();
            // println!("test: {} {}", test, candidates);
            if test < 9 {
                // test candidate field:
                // 1. knock out lowest bit
                // 2. 'recursion'
                cur_candidates.bit_reset(test as usize);
                let edit = board.manipulate(*cur_field, test as usize);
                *cur_edit = edit;

                // stack.push(IterState2::default());
                candidates_stack.push(u16::MAX);
                edit_stack.push(Edit::default());
                field_stack.push(usize::MAX);
            } else {
                // all candidate numbers knocked out but not solved -> return / backtrack
                board.open.push(*cur_field);
                candidates_stack.pop();
                edit_stack.pop();
                field_stack.pop();
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
            let candidates = board.candidates_for(*field);
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
