use std::collections::HashMap;

use crate::{
    execution_tree::{
        Cond, Expr, ExprInner, FnCall, FnDef, FnRet, Id, Literal, Loop, LoopBr, Program, Scope,
        VarAssign, VarCall, VarDef,
    },
    value::{
        function::{
            ConstructedFunctionExecutor, Function, FunctionExecutor, NativeFunctionExecutor,
        },
        Value,
    },
};

pub struct FrameBuilder {
    variables: HashMap<Id, Value>,
}

impl FrameBuilder {
    fn new() -> Self {
        let variables = HashMap::new();
        Self { variables }
    }

    pub fn variable(&mut self, variable_id: &Id, default: Value) {
        self.variables.insert(variable_id.clone(), default);
    }
}

pub struct Frame {
    _scope_id: Id,
    variables: HashMap<Id, Value>,
}

impl Frame {
    /// Puts all variable of that scope in the new frame
    pub fn new<F>(scope: &Scope, builder: F) -> Self
    where
        F: FnOnce(&mut FrameBuilder),
    {
        let Scope {
            scope_id,
            local_variables,
            ..
        } = scope;

        let mut frame_builder = FrameBuilder::new();
        for variable in local_variables {
            frame_builder.variable(variable, Value::None);
        }
        builder(&mut frame_builder);

        let FrameBuilder { variables } = frame_builder;
        let scope_id = scope_id.clone();
        Self {
            _scope_id: scope_id,
            variables,
        }
    }
}

pub struct Stack {
    frames: Vec<Frame>,
}

impl Stack {
    pub fn new() -> Self {
        let frames = Vec::new();
        Self { frames }
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) {
        let _dropped = self.frames.pop();
    }

    pub fn get(&self, variable_id: &Id) -> Option<&Value> {
        self.frames.iter().rev().find_map(|frame| {
            frame
                .variables
                .iter()
                .find(|(id, _value)| **id == *variable_id)
                .map(|(_id, value)| value)
        })
    }

    pub fn get_mut(&mut self, variable_id: &Id) -> Option<&mut Value> {
        self.frames.iter_mut().rev().find_map(|frame| {
            frame
                .variables
                .iter_mut()
                .find(|(id, _value)| **id == *variable_id)
                .map(|(_id, value)| value)
        })
    }
}

pub struct ShortCircuit {
    value: Value,
    destination_scope_id: Id,
}

pub enum ExecReturn {
    Value(Value),
    ShortCircuit(ShortCircuit),
}

impl ExecReturn {
    pub fn new_value(value: Value) -> Self {
        Self::Value(value)
    }

    pub fn new_short_circuit(value: Value, destination_scope_id: Id) -> Self {
        Self::ShortCircuit(ShortCircuit {
            destination_scope_id,
            value,
        })
    }

    pub fn into_value(self) -> Option<Value> {
        if let Self::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_shortcircuit(self) -> Option<ShortCircuit> {
        if let Self::ShortCircuit(short_circuit) = self {
            Some(short_circuit)
        } else {
            None
        }
    }
}

impl Into<ExecReturn> for Value {
    fn into(self) -> ExecReturn {
        ExecReturn::new_value(self)
    }
}

pub struct Runtime {
    stack: Stack,
}

impl Runtime {
    pub fn new() -> Self {
        let stack = Stack::new();
        Self { stack }
    }

    pub fn execute(&mut self, program: &Program) -> Value {
        self.execute_scope(&program.main_scope_id, program, |_| ())
            .into_value()
            .unwrap()
    }

    pub fn execute_scope<F>(
        &mut self,
        scope_id: &Id,
        program: &Program,
        frame_builder: F,
    ) -> ExecReturn
    where
        F: FnOnce(&mut FrameBuilder),
    {
        let scope = program.scopes.get(&scope_id).unwrap();
        let Scope {
            parent_scope_id: _,
            expressions,
            ..
        } = scope;
        let frame = Frame::new(scope, |builder| frame_builder(builder));
        self.stack.push_frame(frame);

        let mut last_expression = Value::None;
        for expression in expressions {
            let returned = self.execute_expression(expression, program);
            match returned {
                ExecReturn::ShortCircuit(short_circuit) => {
                    self.stack.pop_frame();
                    return ExecReturn::ShortCircuit(short_circuit);
                }
                ExecReturn::Value(value) => {
                    last_expression = value;
                }
            }
        }

        self.stack.pop_frame();
        last_expression.into()
    }

    pub fn execute_expression(&mut self, expression: &Expr, program: &Program) -> ExecReturn {
        match expression.inner() {
            ExprInner::Scope(scope_id) => self.execute_scope(scope_id, program, |_| ()),
            ExprInner::Literal(literal) => self.execute_literal(literal),
            ExprInner::VarDef(variable_definition) => {
                self.execute_variable_definition(variable_definition, program)
            }
            ExprInner::VarAssign(variable_assignment) => {
                self.execute_variable_assignment(variable_assignment, program)
            }
            ExprInner::VarCall(variable_call) => self.execute_variable_call(variable_call),
            ExprInner::FnDef(function_definition) => {
                self.execute_function_definition(function_definition)
            }
            ExprInner::FnCall(function_call) => {
                self.execute_function_call(function_call, program).into()
            }
            ExprInner::FnRet(function_return) => {
                self.execute_function_return(function_return, program)
            }
            ExprInner::Loop(loop_) => self.execute_loop(loop_, program),
            ExprInner::LoopBr(loop_break) => self.execute_loop_break(loop_break, program),
            ExprInner::Cond(condition) => self.execute_condition(condition, program),
        }
    }

    pub fn execute_literal(&self, literal: &Literal) -> ExecReturn {
        let Literal(value) = literal;
        ExecReturn::new_value(value.clone())
    }

    pub fn execute_variable_definition(
        &mut self,
        variable_definition: &VarDef,
        program: &Program,
    ) -> ExecReturn {
        let VarDef { variable_id, value } = variable_definition;
        let value = match self.execute_expression(value, program) {
            ExecReturn::ShortCircuit(short_circuit) => {
                return ExecReturn::ShortCircuit(short_circuit);
            }
            ExecReturn::Value(value) => value,
        };
        let variable = self.stack.get_mut(variable_id).unwrap();
        *variable = value.clone();
        value.into()
    }

    pub fn execute_variable_assignment(
        &mut self,
        variable_assignment: &VarAssign,
        program: &Program,
    ) -> ExecReturn {
        let VarAssign { variable_id, value } = variable_assignment;
        let value = match self.execute_expression(value, program) {
            ExecReturn::Value(value) => value,
            ExecReturn::ShortCircuit(short_circuit) => {
                return ExecReturn::ShortCircuit(short_circuit);
            }
        };
        let variable = self.stack.get_mut(variable_id).unwrap();
        *variable = value.clone();
        value.into()
    }

    pub fn execute_variable_call(&mut self, variable_call: &VarCall) -> ExecReturn {
        let VarCall { variable_id } = variable_call;
        let variable = self.stack.get(variable_id).unwrap();
        variable.clone().into()
    }

    pub fn execute_function_definition(&mut self, function_definition: &FnDef) -> ExecReturn {
        let FnDef {
            parameter_ids: argument_ids,
            body_scope_id,
        } = function_definition;
        let value = Function::new_constructed(argument_ids.clone(), body_scope_id.clone());
        let value = Value::Function(value);
        value.into()
    }

    pub fn execute_function_call(
        &mut self,
        function_call: &FnCall,
        program: &Program,
    ) -> ExecReturn {
        let FnCall {
            variable_id,
            arguments,
        } = function_call;

        let mut collector = Vec::new();
        for argument in arguments {
            match self.execute_expression(argument, program) {
                ExecReturn::Value(value) => collector.push(value),
                ExecReturn::ShortCircuit(short_circuit) => {
                    return ExecReturn::ShortCircuit(short_circuit)
                }
            }
        }
        let arguments = collector;

        let function = self
            .stack
            .get(variable_id)
            .unwrap()
            .as_function()
            .expect("calling a non-function variable")
            .clone();
        match function.executor() {
            FunctionExecutor::Constructed(executor) => self
                .execute_constructed_function(arguments, executor, program)
                .into(),
            FunctionExecutor::Native(executor) => {
                self.execute_native_function(arguments, executor).into()
            }
        }
    }

    pub fn execute_constructed_function(
        &mut self,
        arguments: Vec<Value>,
        executor: &ConstructedFunctionExecutor,
        program: &Program,
    ) -> Value {
        let ConstructedFunctionExecutor {
            parameter_ids,
            body_scope_id,
        } = executor;

        let returned = self.execute_scope(body_scope_id, program, |builder| {
            for (id, argument) in parameter_ids.iter().zip(arguments.into_iter()) {
                builder.variable(id, argument)
            }
        });

        match returned {
            ExecReturn::Value(value) => value,
            ExecReturn::ShortCircuit(ShortCircuit {
                value,
                destination_scope_id: _,
            }) => value,
        }
    }

    pub fn execute_native_function(
        &mut self,
        arguments: Vec<Value>,
        executor: &NativeFunctionExecutor,
    ) -> Value {
        (executor.closure)(arguments)
    }

    pub fn execute_function_return(
        &mut self,
        function_return: &FnRet,
        program: &Program,
    ) -> ExecReturn {
        let FnRet {
            value,
            function_scope_id,
        } = function_return;

        let value = match self.execute_expression(value, program) {
            ExecReturn::Value(value) => value,
            ExecReturn::ShortCircuit(ShortCircuit {
                value,
                destination_scope_id: _,
            }) => value,
        };

        ExecReturn::new_short_circuit(value, function_scope_id.clone())
    }

    pub fn execute_loop(&mut self, loop_: &Loop, program: &Program) -> ExecReturn {
        let Loop { body_scope_id } = loop_;

        loop {
            let value = self.execute_scope(body_scope_id, program, |_| ());
            match value {
                ExecReturn::ShortCircuit(ShortCircuit {
                    value,
                    destination_scope_id,
                }) if destination_scope_id == *body_scope_id => return value.into(),
                ExecReturn::ShortCircuit(short_circuit) => {
                    return ExecReturn::ShortCircuit(short_circuit)
                }
                _ => (),
            }
        }
    }

    pub fn execute_loop_break(&mut self, loop_break: &LoopBr, program: &Program) -> ExecReturn {
        let LoopBr {
            value,
            loop_scope_id,
        } = loop_break;

        let value = match self.execute_expression(value, program) {
            ExecReturn::Value(value) => value,
            ExecReturn::ShortCircuit(short_circuit) => {
                return ExecReturn::ShortCircuit(short_circuit)
            }
        };

        ExecReturn::new_short_circuit(value, loop_scope_id.clone())
    }

    pub fn execute_condition(&mut self, condition: &Cond, program: &Program) -> ExecReturn {
        let Cond {
            condition,
            arm_true,
            arm_false,
        } = condition;

        let value = match self.execute_expression(condition, program) {
            ExecReturn::Value(value) => value,
            ExecReturn::ShortCircuit(short_circuit) => {
                return ExecReturn::ShortCircuit(short_circuit)
            }
        };

        if let Value::Bool(boolean) = value {
            if boolean {
                self.execute_expression(arm_true, program)
            } else {
                arm_false
                    .as_ref()
                    .map(|arm_false| self.execute_expression(arm_false, program))
                    .unwrap_or(Value::Bool(false).into())
            }
        } else {
            panic!("non-boolean in condition");
        }
    }
}
