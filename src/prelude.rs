use crate::{
    execution_tree::parser::ParserBuilder,
    value::{function::Function, Value},
};

pub fn std_prelude(builder: &mut ParserBuilder) {
    builder.prelude("print".into(), Function::new_native(1, print).into())
}

fn print(args: Vec<Value>) -> Value {
    let to_print = args.get(0).unwrap();
    println!("{to_print:?}");
    Value::Bool(true)
}
