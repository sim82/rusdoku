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
    candidates_l_stack: [u8; STACK_SIZE],
    candidates_h_stack: [u8; STACK_SIZE],
    num_stack: [u8; STACK_SIZE],
    field_stack: [u8; STACK_SIZE],
    stack_ptr: usize,
    max_depth: usize,
    num_steps: usize,
    min: u8,
    selected_open_field: u8,
    tmp_open_field: u8,
    candidates_l: u8,
    candidates_h: u8,
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
            candidates_l_stack: [CANDIDATES_L_UNDEFINED; STACK_SIZE],
            candidates_h_stack: [CANDIDATES_H_UNDEFINED; STACK_SIZE],
            num_stack: [0u8; STACK_SIZE],
            field_stack: [FIELD_UNDEFINED; STACK_SIZE],
            stack_ptr: 0,
            max_depth: 0,
            num_steps: 0,
            min: 0,
            selected_open_field: 0,
            tmp_open_field: 0,
            candidates_l: 0,
            candidates_h: 0,
        }
    }
}
impl Board {
    pub fn from_line(line: &str) -> Board {
        let mut board = Board::default();
        let mut line = line.chars();
        board.field_stack[0] = 0;
        while board.field_stack[0] < 9 * 9 {
            let c = line.next().expect("line ended early");

            if c.is_numeric() {
                board.num_stack[0] = c.to_digit(10).unwrap() as u8;
                if board.num_stack[0] < 1 {
                    panic!("bad num in input: {}", board.num_stack[0]);
                }
                board.num_stack[0] -= 1;
                board.selected_open_field = 0;
                while board.selected_open_field < board.num_open {
                    if board.open[board.selected_open_field as usize] == board.field_stack[0] {
                        assert!(board.selected_open_field < u8::MAX);
                        board.remove_open_ordered();
                        break;
                    }
                    board.selected_open_field += 1;
                }

                assert!(board.num_stack[0] < 9);
                board.set_field();
            }
            board.field_stack[0] += 1;
        }
        board
    }

    fn set_field(&mut self) {
        let cur_field = self.field_stack[self.stack_ptr];
        let cur_num = self.num_stack[self.stack_ptr];
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
    fn clear_field(&mut self) {
        let cur_field = self.field_stack[self.stack_ptr];
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

    pub fn candidates_for_tmp_field(&mut self) {
        let field = self.open[self.tmp_open_field as usize];
        self.candidates_l = self.h_free_l[F2H[field as usize]]
            & self.v_free_l[F2V[field as usize]]
            & self.b_free_l[F2B[field as usize]];
        self.candidates_h = self.h_free_h[F2H[field as usize]]
            & self.v_free_h[F2V[field as usize]]
            & self.b_free_h[F2B[field as usize]];
    }
    pub fn push_open(&mut self) {
        let field = self.field_stack[self.stack_ptr];
        assert!(self.num_open < 9 * 9);
        self.open[self.num_open as usize] = field;
        self.num_open += 1;
    }
    pub fn remove_open(&mut self) {
        let i = self.selected_open_field;
        assert!(i < self.num_open);
        self.field_stack[self.stack_ptr] = self.open[i as usize];
        self.num_open -= 1;
        self.open[i as usize] = self.open[self.num_open as usize];
    }
    pub fn remove_open_ordered(&mut self) -> u8 {
        let i = self.selected_open_field;
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

    fn select_candidate(&self) -> u8 {
        let low = self.candidates_l_stack[self.stack_ptr];
        let high = self.candidates_h_stack[self.stack_ptr];
        if low != 0 {
            TRAILING_ZEROS[low as usize]
        } else {
            8 + TRAILING_ZEROS[high as usize]
        }
    }
    fn apply_candidate(&mut self) {
        let bit = self.num_stack[self.stack_ptr];
        if bit < 8 {
            bit_reset(&mut self.candidates_l_stack[self.stack_ptr], bit);
        } else {
            bit_reset(&mut self.candidates_h_stack[self.stack_ptr], bit - 8);
        }
    }
    fn select_open_field(&mut self) {
        self.selected_open_field = u8::MAX;
        self.min = u8::MAX;
        self.tmp_open_field = 0;
        // println!("open: {:?}", open_slice);
        while self.tmp_open_field < self.num_open {
            self.candidates_for_tmp_field();
            let num =
                COUNT_ONES[self.candidates_l as usize] + COUNT_ONES[self.candidates_h as usize];
            if num < self.min {
                self.selected_open_field = self.tmp_open_field as u8;
                self.min = num;
                self.candidates_l_stack[self.stack_ptr] = self.candidates_l;
                self.candidates_h_stack[self.stack_ptr] = self.candidates_h;
            }
            // fun fact: this check seems to make it worse... not sure why. There may be bias in the
            // input puzzles to be harder when starting in the top left corner.
            // keep it for consistency.
            if self.min == 1 {
                break;
            }
            self.tmp_open_field += 1;
        }
        if self.selected_open_field == u8::MAX {
            panic!("no minimal candidate found. should be impossible.")
        }
    }
    fn solve(&mut self) -> bool {
        loop {
            self.max_depth = self.max_depth.max(self.stack_ptr + 1);
            self.num_steps += 1;

            if self.candidates_h_stack[self.stack_ptr] == CANDIDATES_H_UNDEFINED {
                if self.num_open == 0 {
                    self.print();
                    println!("max depth: {}, steps: {}", self.max_depth, self.num_steps);
                    return true;
                }
                self.select_open_field();
                self.remove_open();
            } else {
                assert_eq!(
                    self.fields[self.field_stack[self.stack_ptr] as usize],
                    self.num_stack[self.stack_ptr]
                );
                self.clear_field();
            };
            self.num_stack[self.stack_ptr] = self.select_candidate();
            if self.num_stack[self.stack_ptr] < 9 {
                // test candidate field:
                // 1. knock out lowest bit
                // 2. 'recursion'
                // cur_candidates.bit_reset(*cur_num as usize);
                // bit_reset88(cur_candidates_l, cur_candidates_h, *cur_num);
                self.apply_candidate();
                self.set_field();

                self.stack_ptr += 1;
                self.candidates_l_stack[self.stack_ptr] = CANDIDATES_L_UNDEFINED;
                self.candidates_h_stack[self.stack_ptr] = CANDIDATES_H_UNDEFINED;
                self.num_stack[self.stack_ptr] = 0u8;
                self.field_stack[self.stack_ptr] = FIELD_UNDEFINED;
            } else {
                // unsolvable -> return / backtrack
                self.push_open();
                self.stack_ptr -= 1;
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
