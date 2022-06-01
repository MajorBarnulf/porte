use std::{env::args, fs};

pub mod execution_tree;
pub mod prelude;
pub mod runtime;
pub mod syntax_tree;
pub mod value;
pub mod demo;

fn main() {
    use prelude::std_prelude;

    let path = args().nth(1).expect("[error]: usage 'prout <path>'");
    let input = fs::read_to_string(path).expect("file not found");
    let ast_parser = syntax_tree::parser::ParserWrapper::new();
    let parsed = ast_parser.parse(&input).unwrap();
    let executable = execution_tree::parser::Parser::parse(parsed, |b| std_prelude(b));
    let mut runtime = runtime::Runtime::new();
    runtime.execute(&executable);
}

#[test]
fn it_works() {
    use crate::execution_tree::parser::Parser;
    use crate::prelude::std_prelude;
    use crate::runtime::Runtime;
    use crate::syntax_tree::*;

    let ast = Program {
        body: Scope::new(vec![
            Expr::new_variable_definition("a", Expr::new_literal(3.0)),
            Expr::new_variable_assignment("a", Expr::new_literal(6.0)),
            Expr::new_variable_definition(
                "my_print",
                Expr::new_function_definition(
                    vec!["to_print"],
                    Scope::new(vec![
                        Expr::new_variable_definition("a", Expr::new_variable_call("to_print")),
                        Expr::new_function_call("print", vec![Expr::new_variable_call("a")]),
                    ]),
                ),
            ),
            Expr::new_function_call("my_print", vec!["hello, PROUT".into()]),
            Expr::new_function_call("print", vec![Expr::new_variable_call("a")]),
        ]),
    };
    let exec = Parser::parse(ast, |builder| std_prelude(builder));
    println!("\n\n\n-- running: --");
    let _result = Runtime::new().execute(&exec);
}
