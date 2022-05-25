use std::collections::HashMap;

use crate::value::Value;

#[derive(Debug)]
pub struct Program {
    pub main_scope_id: Id,
    pub scopes: HashMap<Id, Scope>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64); // u64::MAX = 18_446_744_073_709_551_615
impl Id {
    pub fn zero() -> Self {
        Self(0)
    }
    pub fn next(self) -> Self {
        let Self(id) = self;
        Self(id + 1)
    }
}

#[derive(Debug)]
pub struct Expr(Box<ExprInner>);

impl Expr {
    pub fn inner(&self) -> &ExprInner {
        &self.0
    }

    pub fn new_scope(scope_id: Id) -> Self {
        Self(Box::new(ExprInner::Scope(scope_id)))
    }

    pub fn new_literal(literal: Literal) -> Self {
        Self(Box::new(ExprInner::Literal(literal)))
    }

    pub fn new_variable_definition(variable_definition: VarDef) -> Self {
        Self(Box::new(ExprInner::VarDef(variable_definition)))
    }

    pub fn new_variable_assignment(variable_assignment: VarAssign) -> Self {
        Self(Box::new(ExprInner::VarAssign(variable_assignment)))
    }

    pub fn new_variable_call(variable_call: VarCall) -> Self {
        Self(Box::new(ExprInner::VarCall(variable_call)))
    }

    pub fn new_function_definition(function_definition: FnDef) -> Self {
        Self(Box::new(ExprInner::FnDef(function_definition)))
    }

    pub fn new_function_call(function_call: FnCall) -> Self {
        Self(Box::new(ExprInner::FnCall(function_call)))
    }

    pub fn new_function_return(function_return: FnRet) -> Self {
        Self(Box::new(ExprInner::FnRet(function_return)))
    }

    pub fn new_loop(loop_: Loop) -> Self {
        Self(Box::new(ExprInner::Loop(loop_)))
    }

    pub fn new_loop_break(loop_break: LoopBr) -> Self {
        Self(Box::new(ExprInner::LoopBr(loop_break)))
    }

    pub fn new_condition(condition: Cond) -> Self {
        Self(Box::new(ExprInner::Cond(condition)))
    }
}

#[derive(Debug)]
pub enum ExprInner {
    Scope(Id),
    Literal(Literal),
    VarDef(VarDef),
    VarAssign(VarAssign),
    VarCall(VarCall),
    FnDef(FnDef),
    FnCall(FnCall),
    FnRet(FnRet),
    Loop(Loop),
    LoopBr(LoopBr),
    Cond(Cond),
}

#[derive(Debug)]
pub struct Scope {
    pub scope_id: Id,
    pub parent_scope_id: Option<Id>,
    pub local_variables: Vec<Id>,
    pub expressions: Vec<Expr>,
}

#[derive(Debug)]
pub struct Literal(pub Value);

#[derive(Debug)]
pub struct VarDef {
    pub variable_id: Id,
    pub value: Expr,
}

#[derive(Debug)]
pub struct VarAssign {
    pub variable_id: Id,
    pub value: Expr,
}

#[derive(Debug)]
pub struct VarCall {
    pub variable_id: Id,
}

#[derive(Debug)]
pub struct FnDef {
    pub parameter_ids: Vec<Id>,
    pub body_scope_id: Id,
}

#[derive(Debug)]
pub struct FnCall {
    pub variable_id: Id,
    pub arguments: Vec<Expr>,
}

#[derive(Debug)]
pub struct FnRet {
    pub value: Expr,
    pub function_scope_id: Id,
}

#[derive(Debug)]
pub struct Loop {
    pub body_scope_id: Id,
}

#[derive(Debug)]
pub struct LoopBr {
    pub value: Expr,
    pub loop_scope_id: Id,
}

#[derive(Debug)]
pub struct Cond {
    pub condition: Expr,
    pub arm_true: Expr,
    pub arm_false: Option<Expr>,
}

pub mod parser;
