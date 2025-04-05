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

#[rustfmt::skip]
const OPEN_INITIAL: [u8; 9 * 9] = [
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10,11,12,13,14,15,16,17,18,19,
    20,21,22,23,24,25,26,27,28,29,
    30,31,32,33,34,35,36,37,38,39,
    40,41,42,43,44,45,46,47,48,49,
    50,51,52,53,54,55,56,57,58,59,
    60,61,62,63,64,65,66,67,68,69,
    70,71,72,73,74,75,76,77,78,79,
    80
];

#[derive(Clone)]
struct Board {
    open: [u8; 9 * 9],
    num_open: u8,
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
        Self {
            open: OPEN_INITIAL,
            num_open: 9 * 9,
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
                    for i in 0..board.num_open {
                        if board.open[i as usize] == field {
                            assert!(i < u8::MAX);
                            board.remove_open_ordered(i);
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
    pub fn push_open(&mut self, field: u8) {
        assert!(self.num_open < 9 * 9);
        self.open[self.num_open as usize] = field;
        self.num_open += 1;
    }
    pub fn remove_open(&mut self, i: u8) -> u8 {
        assert!(i < self.num_open);
        let field = self.open[i as usize];
        self.num_open -= 1;
        self.open[i as usize] = self.open[self.num_open as usize];
        field
    }
    pub fn remove_open_ordered(&mut self, i: u8) -> u8 {
        // purely to keep it exactly equal to 'high level' versions for verification. 6502 port can use swap/remove version.

        assert!(i < self.num_open);
        let field = self.open[i as usize];
        self.open
            .copy_within((i as usize + 1)..(self.num_open as usize), i as usize);

        self.num_open -= 1;
        field
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
                if self.num_open == 0 {
                    self.print();
                    println!("max depth: {}, steps: {}", max_depth, num_steps);
                    return true;
                }
                let mut min_i = u8::MAX;
                let mut min = u32::MAX;
                // println!("open: {:?}", open_slice);
                for i in 0..self.num_open {
                    let field = self.open[i as usize];
                    let (candidates_l, candidates_h) = self.candidates_for(field);
                    // let num = candidates.count_ones();
                    let num = count_ones(candidates_l, candidates_h) as u32;
                    if num < min {
                        min_i = i as u8;
                        min = num;
                        *cur_candidates_l = candidates_l;
                        *cur_candidates_h = candidates_h;
                    }
                    // fun fact: this check seems to make it worse... not sure why. There may be bias in the
                    // input puzzles to be harder when starting in the top left corner.
                    // keep it for consistency.
                    if min == 1 {
                        break;
                    }
                }
                if min_i == u8::MAX {
                    panic!("no minimal candidate found. should be impossible.")
                }
                *cur_field = self.remove_open(min_i);
            } else {
                assert_eq!(self.fields[*cur_field as usize], *cur_num);
                self.clear_field(*cur_field);
            };
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
                self.push_open(*cur_field);
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
    } else {
        bit_reset(high, bit - 8);
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
