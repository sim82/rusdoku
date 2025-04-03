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
    open: Vec<u8>,
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
impl Board {
    pub fn from_line(line: &str) -> Board {
        let mut board = Board::default();
        let mut line = line.chars();
        for i in 0..9 {
            for j in 0..9 {
                let c = line.next().expect("line ended early");

                if c.is_numeric() {
                    let num = c.to_digit(10).unwrap() as u8;
                    if num < 1 {
                        panic!("bad num in input: {}", num);
                    }
                    let num = num - 1;

                    let field = (i * 9 + j) as u8;
                    for i in 0..board.open.len() {
                        if board.open[i] == field {
                            board.open.remove(i);
                            break;
                        }
                    }

                    {
                        let this = &mut board;
                        let num = num;
                        assert!(num < 9);

                        this.h_free[F2H[field as usize]].bit_reset(num as usize);
                        this.v_free[F2V[field as usize]].bit_reset(num as usize);
                        this.b_free[F2B[field as usize]].bit_reset(num as usize);

                        // let f = &mut self.fields[addr.y][addr.x];
                        let f = &mut this.fields[field as usize];
                        assert_eq!(*f, Field::Empty);
                        *f = Field::Set(num as u8);
                        // Edit {
                        //     field,
                        //     num: num as u8,
                        // }
                    };
                }
            }
        }
        board
    }

    pub fn candidates_for(&self, field: u8) -> u16 {
        self.h_free[F2H[field as usize]]
            & self.v_free[F2V[field as usize]]
            & self.b_free[F2B[field as usize]]
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
    fn solve(&mut self) -> bool {
        let mut candidates_stack = [u16::MAX; STACK_SIZE];
        let mut num_stack = [0u8; STACK_SIZE];
        let mut field_stack = [u8::MAX; STACK_SIZE];
        let mut stack_ptr = 0usize; // first element is already correct content

        let mut max_depth: usize = 0;
        let mut num_steps: usize = 0;

        loop {
            max_depth = max_depth.max(stack_ptr + 1);
            num_steps += 1;
            let cur_candidates = &mut candidates_stack[stack_ptr];
            let cur_num = &mut num_stack[stack_ptr];
            let cur_field = &mut field_stack[stack_ptr];

            if *cur_candidates == u16::MAX {
                if self.open.is_empty() {
                    self.print();
                    println!("max depth: {}, steps: {}", max_depth, num_steps);
                    return true;
                }
                *cur_candidates = 0u16;
                // let mut min_candidates = 0u16;
                let mut min_i = usize::MAX;
                let mut min = u32::MAX;
                for (i, field) in self.open.iter().enumerate() {
                    let candidates = self.candidates_for(*field);
                    let num = candidates.count_ones();
                    if num < min {
                        min_i = i;
                        min = num;
                        *cur_candidates = candidates;
                    }
                    if min == 1 {
                        break;
                    }
                }
                if min_i == usize::MAX {
                    panic!("no minimal candidate found. should be impossible.")
                }
                *cur_field = self.open.swap_remove(min_i);
            } else {
                self.h_free[F2H[*cur_field as usize]].bit_set(*cur_num as usize);
                self.v_free[F2V[*cur_field as usize]].bit_set(*cur_num as usize);
                self.b_free[F2B[*cur_field as usize]].bit_set(*cur_num as usize);
                self.fields[*cur_field as usize] = Field::Empty;
            };
            let test = cur_candidates.trailing_zeros() as u8;
            if test < 9 {
                // test candidate field:
                // 1. knock out lowest bit
                // 2. 'recursion'
                cur_candidates.bit_reset(test as usize);
                assert!(test < 9);

                self.h_free[F2H[*cur_field as usize]].bit_reset(test as usize);
                self.v_free[F2V[*cur_field as usize]].bit_reset(test as usize);
                self.b_free[F2B[*cur_field as usize]].bit_reset(test as usize);
                self.fields[*cur_field as usize] = Field::Set(test as u8);

                *cur_num = test;
                stack_ptr += 1;
                candidates_stack[stack_ptr] = u16::MAX;
                num_stack[stack_ptr] = 0u8;
                field_stack[stack_ptr] = u8::MAX;
            } else {
                // unsolvable -> return / backtrack
                stack_ptr -= 1;
                self.open.push(*cur_field);
            }
        }
    }
}
const STACK_SIZE: usize = 9 * 9;

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
            true => {
                println!("solved.");
            }
            false => println!("unsolvable"),
        }
    }
    println!("end");
}
