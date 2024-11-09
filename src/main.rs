use intrusive_collections::intrusive_adapter;
use intrusive_collections::{LinkedList, LinkedListLink};
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::FromIterator;
use std::rc::Rc;
use typenum::consts::U9;

struct Field {
    h_hook: LinkedListLink,
    v_hook: LinkedListLink,
    b_hook: LinkedListLink,

    x: usize,
    y: usize,
}

impl Field {
    pub fn new_rc(x: usize, y: usize) -> Rc<Self> {
        Rc::new(Field {
            h_hook: LinkedListLink::new(),
            v_hook: LinkedListLink::new(),
            b_hook: LinkedListLink::new(),
            x,
            y,
        })
    }
}

type Bitset = fixed_bitset::Bitset<U9>;
type ListV = LinkedList<LinkAdapterV>;
type ListH = LinkedList<LinkAdapterH>;
type ListB = LinkedList<LinkAdapterB>;

struct Board {
    h_free: [Bitset; 9],
    v_free: [Bitset; 9],
    b_free: [Bitset; 9],

    board: Vec<isize>,

    h_fields: [ListH; 9],
    v_fields: [ListV; 9],
    b_fields: [ListB; 9],

    open: Vec<Rc<Field>>,
}

fn get_block(x: usize, y: usize) -> usize {
    (y / 3) * 3 + (x / 3)
}

impl Board {
    pub fn new_rc() -> Rc<RefCell<Board>> {
        Rc::new(RefCell::new(Board {
            h_free: [Bitset::from_iter(0..9); 9],
            v_free: [Bitset::from_iter(0..9); 9],
            b_free: [Bitset::from_iter(0..9); 9],
            board: vec![-1; 9 * 9],

            // TODO: use array-init crate
            h_fields: [
                ListH::default(),
                ListH::default(),
                ListH::default(),
                ListH::default(),
                ListH::default(),
                ListH::default(),
                ListH::default(),
                ListH::default(),
                ListH::default(),
            ],
            v_fields: [
                ListV::default(),
                ListV::default(),
                ListV::default(),
                ListV::default(),
                ListV::default(),
                ListV::default(),
                ListV::default(),
                ListV::default(),
                ListV::default(),
            ],
            b_fields: [
                ListB::default(),
                ListB::default(),
                ListB::default(),
                ListB::default(),
                ListB::default(),
                ListB::default(),
                ListB::default(),
                ListB::default(),
                ListB::default(),
            ],
            open: Vec::new(),
        }))
    }

    pub fn init(&mut self, line: &str) {
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
                    if !(!self.geth(j).contains(num) || !self.getv(i).contains(num)) { !self.getb(j, i).contains(num); }

                    self.manipulate(j, i, num);
                } else {
                    self.open.push(Field::new_rc(j, i));
                }
            }
        }
    }
    pub fn link(&mut self, field: Rc<Field>) {
        let h = &mut self.h_fields[field.x];
        let v = &mut self.v_fields[field.y];
        let b = &mut self.b_fields[get_block(field.x, field.y)];

        h.cursor_mut().insert_after(field.clone());
        v.cursor_mut().insert_after(field.clone());
        b.cursor_mut().insert_after(field.clone());
    }
    pub fn unlink(&mut self, field: Rc<Field>) {
        let h = &mut self.h_fields[field.x];
        let v = &mut self.v_fields[field.y];
        let b = &mut self.b_fields[get_block(field.x, field.y)];

        h.cursor_mut().insert_after(field.clone());
        v.cursor_mut().insert_after(field.clone());
        b.cursor_mut().insert_after(field.clone());
    }

    pub fn geth(&mut self, x: usize) -> &mut Bitset {
        &mut self.h_free[x]
    }
    pub fn getv(&mut self, y: usize) -> &mut Bitset {
        &mut self.v_free[y]
    }
    pub fn getb(&mut self, x: usize, y: usize) -> &mut Bitset {
        &mut self.b_free[get_block(x, y)]
    }
    pub fn get_field(&mut self, x: usize, y: usize) -> &mut isize {
        &mut self.board[get_block(x, y)]
    }

    pub fn manipulate(&mut self, x: usize, y: usize, num: usize) -> (usize, usize, usize) {
        self.geth(x).remove(num);
        self.getv(y).remove(num);
        self.getb(x, y).remove(num);
        *self.get_field(x, y) = num as isize;
        (x, y, num)
    }
    pub fn rollback(&mut self, c: (usize, usize, usize)) {
        let (x, y, num) = c;
        self.geth(x).insert(num);
        self.getv(y).insert(num);
        self.getb(x, y).insert(num);
        *self.get_field(x, y) = -1;
    }
}

intrusive_adapter!(LinkAdapterV = Rc<Field>: Field { v_hook: LinkedListLink });
intrusive_adapter!(LinkAdapterH = Rc<Field>: Field { h_hook: LinkedListLink });
intrusive_adapter!(LinkAdapterB = Rc<Field>: Field { b_hook: LinkedListLink });

fn main() {
    // let mut h_list = ListH::default();
    // let mut v_list = ListV::default();
    // let mut b_list = ListB::default();

    // let mut f = Field::new_rc(0, 0);
    // h_list.cursor_mut().insert_after(f.clone());
    // v_list.cursor_mut().insert_after(f.clone());
    // b_list.cursor_mut().insert_after(f.clone());

    let file = File::open("hardest.txt").unwrap();
    for line in io::BufReader::new(file).lines() {
        let board = Board::new_rc();
        let line = line.unwrap();
        board.borrow_mut().init(&line[..]);
        println!("{:?}", board.borrow().v_free[0]);
    }
}
