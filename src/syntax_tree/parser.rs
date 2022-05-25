use chumsky::prelude::*;

use super::*;

pub fn scope_parser() -> impl Parser<char, Scope, Error = Simple<char>> {
    expression_parser()
        .chain(
            just(';')
                .padded()
                .ignore_then(expression_parser())
                .repeated(),
        )
        .delimited_by(just('{'), just('}'))
        .map(|instructions| Scope { instructions })
}

pub fn literal_value() -> impl Parser<char, Value, Error = Simple<char>> {
    let frac = just('.').chain(text::digits(10));
    let number = just('-')
        .or_not()
        .chain(text::int(10))
        .chain::<char, _, _>(frac.or_not().flatten())
        .collect::<String>()
        .from_str()
        .unwrapped();
    let literal = number.clone().map(|n| Value::Number(n)); // TODO: add other types
    literal
}

pub fn name() -> impl Parser<char, String, Error = Simple<char>> {
    let name = just('a')
        .or(just('b'))
        .or(just('c'))
        .or(just('d'))
        .or(just('e'))
        .or(just('f'))
        .or(just('g'))
        .or(just('h'))
        .or(just('i'))
        .or(just('j'))
        .or(just('k'))
        .or(just('l'))
        .or(just('m'))
        .or(just('n'))
        .or(just('o'))
        .or(just('p'))
        .or(just('q'))
        .or(just('r'))
        .or(just('s'))
        .or(just('t'))
        .or(just('u'))
        .or(just('v'))
        .or(just('w'))
        .or(just('x'))
        .or(just('y'))
        .or(just('z'))
        .or(just('-'))
        .or(just('_'))
        .repeated()
        .map(|v| String::from_iter(&v));
    name
}

pub fn variable_definition_parser() -> impl Parser<char, VarDef, Error = Simple<char>> {
    name()
        .padded()
        .then_ignore(just(":"))
        .padded()
        .then(expression_parser())
        .map(|(name, value)| VarDef { name, value })
}

pub fn variable_assignement_parser() -> impl Parser<char, VarAssign, Error = Simpl<char>> {
	expression_parser().then() name()
}

pub fn expression_parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let scope = scope_parser().map(|s| s.into());
    let litteral = literal_value().map(|v| Expr::new_literal(v));
    let variable_definition = variable_definition_parser().map(|s| s.into());
    let variable_assignment;
    let variable_call;
    let function_definition;
    let function_call;
    let function_return;
    let loop_;
    let loop_break;
    let condition;
    scope
        .or(litteral)
        .or(variable_definition)
        .or(variable_assignment)
        .or(variable_call)
        .or(function_definition)
        .or(function_call)
        .or(function_return)
        .or(loop_)
        .or(loop_break)
        .or(condition)
}

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let scope = expression_parser().chain(
        just(';')
            .padded()
            .ignore_then(expression_parser())
            .repeated(),
    );

    let program = scope
        .map(|instructions| {
            let body = Scope { instructions };
            Program { body }
        })
        .then_ignore(end().recover_with(skip_then_retry_until([])));
    program
}

// impl Parser {
//     pub fn parse(input: &str) -> Program {
//         todo!()
//     }
// }

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
        a - 1 -> a
    }
}

*/
