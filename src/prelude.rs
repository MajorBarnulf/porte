use std::collections::HashMap;

use crate::{
    execution_tree::parser::ParserBuilder,
    value::{function::Function, Value},
};

pub fn std_prelude(builder: &mut ParserBuilder) {
    let functions: Vec<(_, _, fn(Vec<Value>) -> Value)> = vec![
        ("out", 1, out),
        ("add", 2, add),
        ("sub", 2, sub),
        ("eq", 2, eq),
        ("sup", 2, sup),
        ("inf", 2, inf),
        ("and", 2, and),
        ("or", 2, or),
        ("not", 1, not),
        ("str", 2, str),
        ("obj", 0, obj),
        ("set", 3, set),
        ("get", 2, get),
    ];

    for (name, arg_count, closure) in functions {
        builder.prelude(name.into(), Function::new_native(arg_count, closure).into());
    }
}

fn out(args: Vec<Value>) -> Value {
    let to_print = args.get(0).unwrap();
    let str = value_to_string(to_print);
    println!("{str}");
    to_print.clone()
}

fn add(args: Vec<Value>) -> Value {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => (l + r).into(),
        (Value::String(l), Value::String(r)) => format!("{l}{r}").into(),
        (Value::Number(l), Value::String(r)) => format!("{l}{r}").into(),
        (Value::String(l), Value::Number(r)) => format!("{l}{r}").into(),
        _ => unreachable!(),
    }
}

fn sub(args: Vec<Value>) -> Value {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => (l - r).into(),
        _ => panic!("substracting non-numbers"),
    }
}

fn eq(args: Vec<Value>) -> Value {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();
    match (lhs, rhs) {
        (Value::Bool(l), Value::Bool(r)) => (l == r).into(),
        (Value::Number(l), Value::Number(r)) => (l == r).into(),
        (Value::String(l), Value::String(r)) => (l == r).into(),
        _ => panic!("comparing different types"),
    }
}

fn sup(args: Vec<Value>) -> Value {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => (l > r).into(),
        _ => panic!("comparing non-numeric"),
    }
}

fn inf(args: Vec<Value>) -> Value {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();
    match (lhs, rhs) {
        (Value::Number(l), Value::Number(r)) => (l < r).into(),
        _ => panic!("comparing non-numeric"),
    }
}

fn and(args: Vec<Value>) -> Value {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();
    match (lhs, rhs) {
        (Value::Bool(l), Value::Bool(r)) => (*l && *r).into(),
        _ => panic!("intersection of non-boolean"),
    }
}

fn or(args: Vec<Value>) -> Value {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();
    match (lhs, rhs) {
        (Value::Bool(l), Value::Bool(r)) => (*l || *r).into(),
        _ => panic!("union of non-boolean"),
    }
}

fn not(args: Vec<Value>) -> Value {
    let input = args.get(0).unwrap();
    let result = !input.as_bool().expect("complementing non-bool");
    result.into()
}

fn value_to_string(input: &Value) -> String {
    match input {
        Value::None => "None".to_string(),
        Value::Bool(b) => format!("{b}"),
        Value::Number(n) => format!("{n}"),
        Value::String(s) => s.clone(),
        Value::Object(_) => "[object]".into(),
        Value::Function(_) => "[function]".into(),
    }
}

fn str(args: Vec<Value>) -> Value {
    let input = args.get(0).unwrap();
    value_to_string(input).into()
}

fn obj(_: Vec<Value>) -> Value {
    Value::Object(HashMap::new())
}

fn set(mut args: Vec<Value>) -> Value {
    let name = args.get(1).unwrap().as_string().unwrap().to_string();
    let value = args.get(2).unwrap().clone();
    let object = args.get_mut(0).unwrap().as_object_mut().unwrap();
    object.insert(name, value.clone());
    value
}

fn get(args: Vec<Value>) -> Value {
    let object = args.get(0).unwrap().as_object().unwrap();
    let name = args.get(1).unwrap().as_string().unwrap();
    object
        .get(name)
        .map(|value| value.clone())
        .unwrap_or_else(|| false.into())
}
