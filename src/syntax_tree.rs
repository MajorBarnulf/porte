use crate::value::Value;

#[derive(Debug)]
pub struct Program {
    pub body: Scope,
}

#[derive(Debug)]
pub struct Expr(pub Box<ExprInner>);

impl Expr {
    pub fn new(inner: ExprInner) -> Self {
        Self(Box::new(inner))
    }

    pub fn inner(&self) -> &ExprInner {
        let Self(inner) = self;
        inner
    }

    pub fn into_inner(self) -> ExprInner {
        let Self(inner) = self;
        *inner
    }

    pub fn new_scope(instructions: Vec<Expr>) -> Self {
        Self(Box::new(ExprInner::Scope(Scope { instructions })))
    }

    pub fn new_literal<V: Into<Value>>(value: V) -> Self {
        Self(Box::new(ExprInner::Literal(Literal(value.into()))))
    }

    pub fn new_variable_definition<S: ToString>(name: S, value: Expr) -> Self {
        let name = name.to_string();
        Self(Box::new(ExprInner::VarDef(VarDef { name, value })))
    }

    pub fn new_variable_assignment<S: ToString>(name: S, value: Expr) -> Self {
        let name = name.to_string();
        Self(Box::new(ExprInner::VarAssign(VarAssign { name, value })))
    }

    pub fn new_variable_call<S: ToString>(name: S) -> Self {
        let name = name.to_string();
        Self(Box::new(ExprInner::VarCall(VarCall { name })))
    }

    pub fn new_function_definition<S: ToString>(parameter_names: Vec<S>, body: Scope) -> Self {
        let parameter_names = parameter_names.into_iter().map(|s| s.to_string()).collect();
        Self(Box::new(ExprInner::FnDef(FnDef {
            body,
            parameter_names,
        })))
    }

    pub fn new_function_call<S: ToString>(name: S, arguments: Vec<Expr>) -> Self {
        let name = name.to_string();
        Self(Box::new(ExprInner::FnCall(FnCall { name, arguments })))
    }

    pub fn new_function_return(value: Expr) -> Self {
        Self(Box::new(ExprInner::FnRet(FnRet { value })))
    }

    pub fn new_loop(body: Scope) -> Self {
        Self(Box::new(ExprInner::Loop(Loop { body })))
    }

    pub fn new_loop_break(value: Expr) -> Self {
        Self(Box::new(ExprInner::LoopBr(LoopBr { value })))
    }

    pub fn new_condition(condition: Expr, arm_true: Expr, arm_false: Option<Expr>) -> Self {
        Self(Box::new(ExprInner::Cond(Cond {
            condition,
            arm_true,
            arm_false,
        })))
    }
}

impl From<Scope> for Expr {
    fn from(input: Scope) -> Self {
        Self::new(ExprInner::Scope(input))
    }
}

impl From<Literal> for Expr {
    fn from(input: Literal) -> Self {
        Self::new(ExprInner::Literal(input))
    }
}

impl From<VarDef> for Expr {
    fn from(input: VarDef) -> Self {
        Self::new(ExprInner::VarDef(input))
    }
}

impl From<VarAssign> for Expr {
    fn from(input: VarAssign) -> Self {
        Self::new(ExprInner::VarAssign(input))
    }
}

impl From<VarCall> for Expr {
    fn from(input: VarCall) -> Self {
        Self::new(ExprInner::VarCall(input))
    }
}

impl From<FnDef> for Expr {
    fn from(input: FnDef) -> Self {
        Self::new(ExprInner::FnDef(input))
    }
}

impl From<FnCall> for Expr {
    fn from(input: FnCall) -> Self {
        Self::new(ExprInner::FnCall(input))
    }
}

impl From<FnRet> for Expr {
    fn from(input: FnRet) -> Self {
        Self::new(ExprInner::FnRet(input))
    }
}

impl From<Loop> for Expr {
    fn from(input: Loop) -> Self {
        Self::new(ExprInner::Loop(input))
    }
}

impl From<LoopBr> for Expr {
    fn from(input: LoopBr) -> Self {
        Self::new(ExprInner::LoopBr(input))
    }
}

impl From<Cond> for Expr {
    fn from(input: Cond) -> Self {
        Self::new(ExprInner::Cond(input))
    }
}

impl<T> From<T> for Expr
where
    T: Into<Value>,
{
    fn from(input: T) -> Self {
        Self::new_literal(input.into())
    }
}

#[derive(Debug)]
pub enum ExprInner {
    Scope(Scope),
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
    pub instructions: Vec<Expr>,
}

impl Scope {
    pub fn new(instructions: Vec<Expr>) -> Self {
        Self { instructions }
    }
}

#[derive(Debug)]
pub struct Literal(pub Value);

#[derive(Debug)]
pub struct VarDef {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug)]
pub struct VarAssign {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug)]
pub struct VarCall {
    pub name: String,
}

#[derive(Debug)]
pub struct FnDef {
    pub parameter_names: Vec<String>,
    pub body: Scope,
}

#[derive(Debug)]
pub struct FnCall {
    pub name: String,
    pub arguments: Vec<Expr>,
}

#[derive(Debug)]
pub struct FnRet {
    pub value: Expr,
}

#[derive(Debug)]
pub struct Loop {
    pub body: Scope,
}

#[derive(Debug)]
pub struct LoopBr {
    pub value: Expr,
}

#[derive(Debug)]
pub struct Cond {
    pub condition: Expr,
    pub arm_true: Expr,
    pub arm_false: Option<Expr>,
}

pub mod parser;
