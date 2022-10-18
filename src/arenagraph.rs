// TODO: Don't forget to remove
#![allow(dead_code)]

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use slab::Slab;

#[derive(Debug, Clone, Copy)]
pub enum BinaryOpType {
    Sum,
    Mul,
}

#[derive(Debug)]
pub enum ArenaNode<T> {
    Input {
        name: &'static str,
        value: Rc<Cell<f32>>,
        deps: Rc<RefCell<Vec<T>>>,
    },
    BinaryOp {
        lhs: T,
        rhs: T,
        cache: Cell<f32>,
        op_type: BinaryOpType,
    },
}

#[derive(Debug)]
pub struct ArenaGraph {
    elems: Slab<ArenaNode<usize>>,
}
