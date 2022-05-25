/*

use std::collections::HashMap;

use crate::{abstract_tree::Id, value::Value};

pub struct Program {
    statements: Vec<Statement>,
    state: State,
}

pub enum Statement {
    Label(Label),
    VarAssign(VarAssign),
    FnCall(FnCall),
    Branch(Branch),
}

pub struct VarAssign {
    variable:
}

pub struct FnCall {}

pub struct State {
    variables: HashMap<Id, Vec<Value>>,
    next_instruction_index: usize,
}

impl Statement {
    pub fn run(&self, state: &mut State) {}
}

pub mod parser {
    pub struct Parser;

    impl Parser {
        pub fn parse() {
            //
        }
    }
}
 */
