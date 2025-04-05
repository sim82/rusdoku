use std::env::args;
use std::fs::File;
use std::io::{self, BufRead};

const STACK_SIZE: usize = 9 * 9;
const FIELD_UNDEFINED: u8 = u8::MAX;
const CANDIDATES_L_UNDEFINED: u8 = 0b00000000;
const CANDIDATES_H_UNDEFINED: u8 = 0b10000000;

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

#[rustfmt::skip]
const COUNT_ONES : [u8; 256] = [
    0,1,1,2,1,2,2,3,1,2,2,3,2,3,3,4, 
    1,2,2,3,2,3,3,4,2,3,3,4,3,4,4,5, 
    1,2,2,3,2,3,3,4,2,3,3,4,3,4,4,5, 
    2,3,3,4,3,4,4,5,3,4,4,5,4,5,5,6, 
    1,2,2,3,2,3,3,4,2,3,3,4,3,4,4,5, 
    2,3,3,4,3,4,4,5,3,4,4,5,4,5,5,6, 
    2,3,3,4,3,4,4,5,3,4,4,5,4,5,5,6, 
    3,4,4,5,4,5,5,6,4,5,5,6,5,6,6,7, 
    1,2,2,3,2,3,3,4,2,3,3,4,3,4,4,5, 
    2,3,3,4,3,4,4,5,3,4,4,5,4,5,5,6, 
    2,3,3,4,3,4,4,5,3,4,4,5,4,5,5,6, 
    3,4,4,5,4,5,5,6,4,5,5,6,5,6,6,7,
    2,3,3,4,3,4,4,5,3,4,4,5,4,5,5,6, 
    3,4,4,5,4,5,5,6,4,5,5,6,5,6,6,7, 
    3,4,4,5,4,5,5,6,4,5,5,6,5,6,6,7, 
    4,5,5,6,5,6,6,7,5,6,6,7,6,7,7,8,
];

#[rustfmt::skip]
const TRAILING_ZEROS: [u8; 256] = [
    8,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    6,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    7,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    6,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
];

#[rustfmt::skip]
const SET_MASK: [u8; 8] = [
    0b00000001,
    0b00000010,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b01000000,
    0b10000000,
];

#[rustfmt::skip]
const RESET_MASK: [u8; 8] = [
    0b11111110,
    0b11111101,
    0b11111011,
    0b11110111,
    0b11101111,
    0b11011111,
    0b10111111,
    0b01111111,
];

#[derive(Clone)]
struct Board {
    open: Vec<u8>,
    fields: [u8; 9 * 9],
    h_free_l: [u8; 9],
    v_free_l: [u8; 9],
    b_free_l: [u8; 9],
    h_free_h: [u8; 9],
    v_free_h: [u8; 9],
    b_free_h: [u8; 9],
}
impl Default for Board {
    fn default() -> Self {
        // open.reverse();
        Self {
            // open: (0..9)
            //     .flat_map(|y| (0..9).map(move |x| y * 9 + x))
            //     .collect(),
            open: (0..(9 * 9)).collect(),
            fields: [FIELD_UNDEFINED; 9 * 9],
            h_free_l: [0b11111111; 9],
            v_free_l: [0b11111111; 9],
            b_free_l: [0b11111111; 9],
            h_free_h: [0b00000001; 9],
            v_free_h: [0b00000001; 9],
            b_free_h: [0b00000001; 9],
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

                    assert!(num < 9);
                    board.set_field(field, num);
                }
            }
        }
        board
    }

    fn set_field(&mut self, cur_field: u8, cur_num: u8) {
        assert_eq!(self.fields[cur_field as usize], FIELD_UNDEFINED);
        if cur_num < 8 {
            bit_reset(&mut self.h_free_l[F2H[cur_field as usize]], cur_num);
            bit_reset(&mut self.v_free_l[F2V[cur_field as usize]], cur_num);
            bit_reset(&mut self.b_free_l[F2B[cur_field as usize]], cur_num);
        } else {
            let cur_num = cur_num - 8;
            bit_reset(&mut self.h_free_h[F2H[cur_field as usize]], cur_num);
            bit_reset(&mut self.v_free_h[F2V[cur_field as usize]], cur_num);
            bit_reset(&mut self.b_free_h[F2B[cur_field as usize]], cur_num);
        }
        self.fields[cur_field as usize] = cur_num;
    }
    fn clear_field(&mut self, cur_field: u8) {
        assert_ne!(self.fields[cur_field as usize], FIELD_UNDEFINED);
        let cur_num = self.fields[cur_field as usize];
        if cur_num < 8 {
            bit_set(&mut self.h_free_l[F2H[cur_field as usize]], cur_num);
            bit_set(&mut self.v_free_l[F2V[cur_field as usize]], cur_num);
            bit_set(&mut self.b_free_l[F2B[cur_field as usize]], cur_num);
        } else {
            let cur_num = cur_num - 8;
            bit_set(&mut self.h_free_h[F2H[cur_field as usize]], cur_num);
            bit_set(&mut self.v_free_h[F2V[cur_field as usize]], cur_num);
            bit_set(&mut self.b_free_h[F2B[cur_field as usize]], cur_num);
        }
        self.fields[cur_field as usize] = FIELD_UNDEFINED;
    }

    pub fn candidates_for(&self, field: u8) -> (u8, u8) {
        (
            self.h_free_l[F2H[field as usize]]
                & self.v_free_l[F2V[field as usize]]
                & self.b_free_l[F2B[field as usize]],
            self.h_free_h[F2H[field as usize]]
                & self.v_free_h[F2V[field as usize]]
                & self.b_free_h[F2B[field as usize]],
        )
    }
    pub fn print(&self) {
        for y in 0..9 {
            for x in 0..9 {
                match self.fields[y * 9 + x] {
                    FIELD_UNDEFINED => print!(". "),
                    num => print!("{} ", num + 1),
                }
            }
            println!();
        }
    }
    fn solve(&mut self) -> bool {
        // let mut candidates_stack = [CANDIDATES_UNDEFINED; STACK_SIZE];
        let mut candidates_l_stack = [CANDIDATES_L_UNDEFINED; STACK_SIZE];
        let mut candidates_h_stack = [CANDIDATES_H_UNDEFINED; STACK_SIZE];
        let mut num_stack = [0u8; STACK_SIZE];
        let mut field_stack = [FIELD_UNDEFINED; STACK_SIZE];
        let mut stack_ptr = 0usize; // first element is already correct content

        let mut max_depth: usize = 0;
        let mut num_steps: usize = 0;

        loop {
            max_depth = max_depth.max(stack_ptr + 1);
            num_steps += 1;
            let cur_candidates_l = &mut candidates_l_stack[stack_ptr];
            let cur_candidates_h = &mut candidates_h_stack[stack_ptr];
            let cur_num = &mut num_stack[stack_ptr];
            let cur_field = &mut field_stack[stack_ptr];

            if *cur_candidates_h == CANDIDATES_H_UNDEFINED {
                if self.open.is_empty() {
                    self.print();
                    println!("max depth: {}, steps: {}", max_depth, num_steps);
                    return true;
                }
                // *cur_candidates = (0u8, 0u8);
                // let mut min_candidates = 0u16;
                let mut min_i = usize::MAX;
                let mut min = u32::MAX;
                for (i, field) in self.open.iter().enumerate() {
                    let (candidates_l, candidates_h) = self.candidates_for(*field);
                    // let num = candidates.count_ones();
                    let num = count_ones(candidates_l, candidates_h) as u32;
                    if num < min {
                        min_i = i;
                        min = num;
                        // *cur_candidates = candidates;
                        *cur_candidates_l = candidates_l;
                        *cur_candidates_h = candidates_h;
                    }
                    // fun fact: this check seems to make it worse... not sure why. There may be bias in the
                    // input puzzles to be harder when starting in the top left corner.
                    if min == 1 {
                        break;
                    }
                }
                if min_i == usize::MAX {
                    panic!("no minimal candidate found. should be impossible.")
                }
                *cur_field = self.open.swap_remove(min_i);
            } else {
                assert_eq!(self.fields[*cur_field as usize], *cur_num);
                self.clear_field(*cur_field);
            };
            // *cur_num = cur_candidates.trailing_zeros() as u8;
            *cur_num = trailing_zeros(*cur_candidates_l, *cur_candidates_h);
            if *cur_num < 9 {
                // test candidate field:
                // 1. knock out lowest bit
                // 2. 'recursion'
                // cur_candidates.bit_reset(*cur_num as usize);
                bit_reset88(cur_candidates_l, cur_candidates_h, *cur_num);
                self.set_field(*cur_field, *cur_num);

                stack_ptr += 1;
                candidates_l_stack[stack_ptr] = CANDIDATES_L_UNDEFINED;
                candidates_h_stack[stack_ptr] = CANDIDATES_H_UNDEFINED;
                num_stack[stack_ptr] = 0u8;
                field_stack[stack_ptr] = FIELD_UNDEFINED;
            } else {
                // unsolvable -> return / backtrack
                stack_ptr -= 1;
                self.open.push(*cur_field);
            }
        }
    }
}

fn bit_set(v: &mut u8, bit: u8) {
    *v |= SET_MASK[bit as usize]
}
fn bit_reset(v: &mut u8, bit: u8) {
    *v &= RESET_MASK[bit as usize]
}
fn bit_reset88(low: &mut u8, high: &mut u8, bit: u8) {
    if bit < 8 {
        bit_reset(low, bit);
        // low.bit_reset(bit as usize);
    } else {
        bit_reset(high, bit - 8);
        // high.bit_reset(bit as usize - 8);
    }
}
fn trailing_zeros(low: u8, high: u8) -> u8 {
    if low != 0 {
        TRAILING_ZEROS[low as usize]
    } else {
        8 + TRAILING_ZEROS[high as usize]
    }
}

fn count_ones(low: u8, high: u8) -> u8 {
    COUNT_ONES[low as usize] + COUNT_ONES[high as usize]
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
    #[test]
    fn gen_count_ones() {
        let out = (0u8..=255u8).map(|i| i.count_ones()).collect::<Vec<_>>();
        println!("{:?}", out);
    }
    #[test]
    fn gen_trailing_zeros() {
        let out = (0u8..=255u8)
            .map(|i| i.trailing_zeros())
            .collect::<Vec<_>>();
        println!("{:?}", out);
    }
}
