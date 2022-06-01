use std::collections::HashMap;

use self::function::Function;
pub mod function {
    use crate::execution_tree::Id;

    use super::Value;

    #[derive(Debug, Clone)]
    pub struct ConstructedFunctionExecutor {
        pub parameter_ids: Vec<Id>,
        pub body_scope_id: Id,
    }

    #[derive(Debug, Clone)]
    pub struct NativeFunctionExecutor {
        pub closure: fn(Vec<Value>) -> Value,
    }

    #[derive(Debug, Clone)]
    pub enum FunctionExecutor {
        Constructed(ConstructedFunctionExecutor),
        Native(NativeFunctionExecutor),
    }

    #[derive(Debug, Clone)]
    pub struct Function {
        pub argument_count: usize,
        pub executor: FunctionExecutor,
    }

    impl Function {
        pub fn new_constructed(argument_ids: Vec<Id>, body_scope_id: Id) -> Self {
            let argument_count = argument_ids.len();
            let executor = FunctionExecutor::Constructed(ConstructedFunctionExecutor {
                parameter_ids: argument_ids,
                body_scope_id,
            });
            Self {
                argument_count,
                executor,
            }
        }

        pub fn new_native(argument_count: usize, closure: fn(Vec<Value>) -> Value) -> Self {
            let executor = FunctionExecutor::Native(NativeFunctionExecutor { closure });
            Self {
                argument_count,
                executor,
            }
        }

        pub fn executor(&self) -> &FunctionExecutor {
            &self.executor
        }
    }

    impl Into<Value> for Function {
        fn into(self) -> Value {
            Value::Function(self)
        }
    }
}
#[derive(Debug, Clone)]
pub enum Value {
    None,
    Bool(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, Value>),
    Function(Function),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Self::Object(h) => Some(h),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
        match self {
            Self::Object(h) => Some(h),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        match self {
            Self::Function(function) => Some(function),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        value.map_or(Value::None, |v| v.into())
    }
}
