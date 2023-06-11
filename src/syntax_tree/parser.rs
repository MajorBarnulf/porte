use super::*;
use chumsky::{prelude::*, text::whitespace};

pub trait AbstractParser<T>: Parser<char, T, Error = Simple<char>> {}

impl<T, U: Parser<char, T, Error = Simple<char>>> AbstractParser<T> for U {}

pub fn debugging_expression_parser() -> impl AbstractParser<Expr> + Clone {
    variable_call_parser().padded().map(|v| v.into())
}

pub fn scope_parser(expression: impl AbstractParser<Expr> + Clone) -> impl AbstractParser<Scope> {
    let open_list = expression.separated_by(just(';'));
    let closed_list = open_list.clone().then_ignore(just(';').then(whitespace()));
    open_list
        .map(|instructions| Scope { instructions })
        .or(closed_list.map(|mut instructions| {
            instructions.push(Value::None.into());
            Scope { instructions }
        }))
        .padded()
        .delimited_by(just('{'), just('}'))
}

#[test]
fn test_scope_parser() {
    let parser = scope_parser(debugging_expression_parser());
    let value = parser.parse("{ arbre }");
    dbg!(value.unwrap());
}

// TODO: add objects ?
pub fn literal_value() -> impl AbstractParser<Value> {
    let bool = just("false")
        .map(|_| false.into())
        .or(just("true").map(|_| true.into()));

    let none = just("none").map(|_| Value::None);

    let string = just('\"')
        .ignore_then(none_of("\"").repeated())
        .then_ignore(just('\"'))
        .map(|v| String::from_iter(&v).into());

    let frac = just('.').chain(text::digits(10));
    let number = just('-')
        .or_not()
        .chain(text::int(10))
        .chain::<char, _, _>(frac.or_not().flatten())
        .collect::<String>()
        .from_str()
        .unwrapped()
        .map(|n: f64| n.into());

    bool.or(none).or(string).or(number)
}

#[test]
fn test_literal_value() {
    let parser = literal_value();
    let value = parser.parse("5");
    assert_eq!(value.unwrap().as_number().unwrap(), 5.);
}

pub fn name() -> impl AbstractParser<String> + Clone {
    let first = one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let rest = first.clone().or(one_of("1234567890-_+/*"));

    first.then(rest.repeated()).map(|(f, v)| {
        let rest = String::from_iter(&v);
        format!("{f}{rest}")
    })
}

pub fn variable_definition_parser(
    expression: impl AbstractParser<Expr>,
) -> impl AbstractParser<VarDef> {
    name()
        .then_ignore(just(":").padded())
        .then(expression)
        .map(|(name, value)| VarDef { name, value })
}

#[test]
fn test_variable_definition_parser() {
    let parser = variable_definition_parser(debugging_expression_parser());
    let value = parser.parse("a : b");
    dbg!(value.unwrap());
}

pub fn variable_assignement_parser(
    expression: impl AbstractParser<Expr>,
) -> impl AbstractParser<VarAssign> {
    name()
        .then_ignore(just("<-").padded())
        .then(expression)
        .map(|(name, value)| VarAssign { name, value })
}

#[test]
fn test_variable_assignement_parser() {
    let parser = variable_assignement_parser(debugging_expression_parser());
    let value = parser.parse("arbre <- expr");
    dbg!(value.unwrap());
}

pub fn variable_call_parser() -> impl AbstractParser<VarCall> + Clone {
    name().map(|name| VarCall { name })
}

#[test]
fn test_variable_call_parser() {
    let parser = variable_call_parser();
    let value = parser.parse("arbre");
    assert_eq!(value.unwrap().name, "arbre".to_string());
}

pub fn function_definition_parser(
    expression: impl AbstractParser<Expr> + Clone,
) -> impl AbstractParser<FnDef> {
    let parameters = name().separated_by(just(',').padded());
    let body = scope_parser(expression);
    parameters
        .padded()
        .delimited_by(just('('), just(')'))
        .then_ignore(just("=>").padded())
        .then(body)
        .map(|(parameter_names, body)| FnDef {
            body,
            parameter_names,
        })
}

#[test]
fn test_function_definition_parser() {
    let parser = function_definition_parser(debugging_expression_parser());
    let value = parser.parse("(a) => { b }");
    dbg!(value.unwrap());
}

pub fn function_call_parser(expression: impl AbstractParser<Expr>) -> impl AbstractParser<FnCall> {
    let parameters = expression.separated_by(just(',').padded());
    name()
        .then(
            parameters
                .padded()
                .delimited_by(just('('), just(')'))
                .padded(),
        )
        .map(|(name, arguments)| FnCall { arguments, name })
}

#[test]
fn test_function_call_parser() {
    let parser = function_call_parser(debugging_expression_parser());
    let value = parser.parse("f( a , b )");
    dbg!(value.unwrap());
}

pub fn function_return_parser(expression: impl AbstractParser<Expr>) -> impl AbstractParser<FnRet> {
    just("return")
        .ignore_then(expression.or_not())
        .map(|expr| expr.unwrap_or(From::<Value>::from(true.into())))
        .map(|value| FnRet { value })
}

#[test]
fn test_function_return_parser() {
    let parser = function_return_parser(debugging_expression_parser());
    let value = parser.parse("return  a");
    dbg!(value.unwrap());
}

pub fn loop_parser(expression: impl AbstractParser<Expr> + Clone) -> impl AbstractParser<Loop> {
    just("loop")
        .then(whitespace())
        .ignore_then(scope_parser(expression))
        .map(|body| Loop { body })
}

#[test]
fn test_loop_parser() {
    let parser = loop_parser(debugging_expression_parser());
    let value = parser.parse("loop { a}");
    dbg!(value.unwrap());
}

pub fn loop_break_parser(expression: impl AbstractParser<Expr>) -> impl AbstractParser<LoopBr> {
    just("break")
        .ignore_then(just(" ").then(whitespace()).ignore_then(expression))
        .map(|value| LoopBr { value })
}

#[test]
fn test_loop_break_parser() {
    let parser = loop_break_parser(debugging_expression_parser());
    let value = parser.parse("break  a");
    dbg!(value.unwrap());
}

pub fn condition_parser(
    expression: impl AbstractParser<Expr> + Clone,
) -> impl AbstractParser<Cond> {
    just("if ")
        .ignore_then(expression.clone())
        .then(expression.clone())
        .then(just("else ").ignore_then(expression).or_not())
        .map(|((condition, arm_true), arm_false)| Cond {
            condition,
            arm_true,
            arm_false,
        })
}

#[test]
fn test_condition_parser() {
    let parser = condition_parser(debugging_expression_parser());
    let value = parser.parse("if a t else f");
    dbg!(value.unwrap());
}

fn _sugar_parser(expression: impl AbstractParser<Expr> + Clone) -> impl AbstractParser<Expr> {
    let add = expression
        .clone()
        .then_ignore(just(" + "))
        .then(expression.clone())
        .map(|(l, r)| {
            FnCall {
                name: "add".into(),
                arguments: vec![l, r],
            }
            .into()
        });

    let sub = expression
        .clone()
        .then_ignore(just(" - "))
        .then(expression.clone())
        .map(|(l, r)| {
            FnCall {
                name: "sub".into(),
                arguments: vec![l, r],
            }
            .into()
        });

    let eq = expression
        .clone()
        .then_ignore(just(" == "))
        .then(expression.clone())
        .map(|(l, r)| {
            FnCall {
                name: "eq".into(),
                arguments: vec![l, r],
            }
            .into()
        });

    let sup = expression
        .clone()
        .then_ignore(just(" > "))
        .then(expression.clone())
        .map(|(l, r)| {
            FnCall {
                name: "sup".into(),
                arguments: vec![l, r],
            }
            .into()
        });

    let inf = expression
        .clone()
        .then_ignore(just(" < "))
        .then(expression.clone())
        .map(|(l, r)| {
            FnCall {
                name: "inf".into(),
                arguments: vec![l, r],
            }
            .into()
        });

    let or = expression
        .clone()
        .then_ignore(just(" || "))
        .then(expression.clone())
        .map(|(l, r)| {
            FnCall {
                name: "or".into(),
                arguments: vec![l, r],
            }
            .into()
        });

    let and = expression
        .clone()
        .then_ignore(just(" && "))
        .then(expression)
        .map(|(l, r)| {
            FnCall {
                name: "and".into(),
                arguments: vec![l, r],
            }
            .into()
        });

    eq.or(add).or(sub).or(sup).or(inf).or(or).or(and)
}

pub fn expression_parser() -> impl AbstractParser<Expr> {
    let expression = recursive(|expression| {
        //let sugar = sugar_parser(expression.clone());
        let condition = condition_parser(expression.clone()).map(|i| i.into());
        let function_definition = function_definition_parser(expression.clone()).map(|i| i.into());
        let function_return = function_return_parser(expression.clone()).map(|i| i.into());
        let loop_ = loop_parser(expression.clone()).map(|i| i.into());
        let loop_break = loop_break_parser(expression.clone()).map(|i| i.into());
        let scope = scope_parser(expression.clone()).map(|i| i.into());
        let litteral = literal_value().map(Expr::new_literal);
        let variable_definition = variable_definition_parser(expression.clone()).map(|i| i.into());
        let variable_assignment = variable_assignement_parser(expression.clone()).map(|i| i.into());
        let function_call = function_call_parser(expression.clone()).map(|i| i.into());
        let variable_call = variable_call_parser().map(|i| i.into());

        //sugar
        //    .or(condition)
        condition
            .or(function_definition)
            .or(function_return)
            .or(loop_)
            .or(loop_break)
            .or(scope)
            .or(litteral)
            .or(variable_definition)
            .or(variable_assignment)
            .or(function_call)
            .or(variable_call)
            .padded()
    });
    expression
}

#[test]
fn test_expression_parser() {
    let text = r##"if a b"##;
    let parser = expression_parser();
    let value = parser.parse(text);
    dbg!(value.unwrap());
}

fn parser() -> impl AbstractParser<Program> {
    let scope = expression_parser()
        .separated_by(just(';').padded())
        .then_ignore(just(';').padded().or_not())
        .then_ignore(end());

    scope.map(|instructions| {
        let body = Scope { instructions };
        Program { body }
    })
}

#[test]
fn test_parser() {
    let example = r##"
print(3);

a: 3;

b: (a) => {
    print("a")
};

f(a)

"##;
    let parser = parser();
    let e = parser.parse(example);
    dbg!(e.unwrap());
}

pub struct ParserWrapper {
    inner: Box<dyn AbstractParser<Program>>,
}

impl ParserWrapper {
    pub fn new() -> Self {
        let inner = Box::new(parser());
        ParserWrapper { inner }
    }

    pub fn parse(&self, input: &str) -> Result<Program, Vec<Simple<char>>> {
        self.inner.parse(input)
    }
}

impl Default for ParserWrapper {
    fn default() -> Self {
        Self::new()
    }
}

/*
// example

my-print: (input) => {
    concatenated: {
        "now I would like to interject for a moment" + input
    };
    print(input);
}

a: 5.2;
loop {
    if a <= 0 {
        break true
    } else {
        a <- a - 1
    }
}

*/
