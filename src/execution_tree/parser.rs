use std::{collections::HashMap, rc::Rc, sync::Mutex};

use crate::{
    execution_tree::{self, Id},
    syntax_tree,
    value::Value,
};

pub struct ParserBuilder {
    prelude: Vec<(String, Value)>,
}

impl ParserBuilder {
    fn new() -> Self {
        let prelude = Vec::new();
        Self { prelude }
    }

    pub fn prelude(&mut self, name: String, value: Value) {
        self.prelude.push((name, value));
    }
}

pub struct Parser {
    scopes: HashMap<Id, execution_tree::Scope>,
}

impl Parser {
    pub fn parse<F>(ast: syntax_tree::Program, builder: F) -> execution_tree::Program
    where
        F: FnOnce(&mut ParserBuilder),
    {
        let syntax_tree::Program { mut body } = ast;

        let mut parser = Self {
            scopes: HashMap::new(),
        };
        let parser_scope = ParserScope::new_root();
        let mut parser_builder = ParserBuilder::new();
        builder(&mut parser_builder);
        for (name, value) in parser_builder.prelude.into_iter().rev() {
            let value = syntax_tree::Expr::new_literal(value);
            let expression = syntax_tree::Expr::new_variable_definition(name, value);
            body.instructions.insert(0, expression);
        }

        let body_scope_id = parser.parse_ast_scope(body, &parser_scope);

        let Self { scopes } = parser;
        execution_tree::Program {
            main_scope_id: body_scope_id,
            scopes,
        }
    }

    pub fn parse_ast_scope(
        &mut self,
        ast_scope: syntax_tree::Scope,
        parser_scope: &ParserScope,
    ) -> execution_tree::Id {
        let syntax_tree::Scope { instructions } = ast_scope;

        let scope_id = parser_scope.get_current_id();
        let parent_scope_id = parser_scope.get_parent_id();
        let expressions = instructions
            .into_iter()
            .map(|expression| self.parse_expression(expression, &parser_scope))
            .collect();
        let local_variables = parser_scope.local_variable_ids();

        let scope = execution_tree::Scope {
            parent_scope_id,
            scope_id,
            expressions,
            local_variables,
        };
        self.scopes.insert(scope_id, scope);
        scope_id
    }

    pub fn parse_expression(
        &mut self,
        expression: syntax_tree::Expr,
        parser_scope: &ParserScope,
    ) -> execution_tree::Expr {
        let inner = expression.into_inner();
        match inner {
            syntax_tree::ExprInner::Scope(scope) => {
                let parser_scope = parser_scope.make_child_common();
                let scope = self.parse_ast_scope(scope, &parser_scope);
                execution_tree::Expr::new_scope(scope)
            }
            syntax_tree::ExprInner::Literal(literal) => {
                let literal = self.parse_literal(literal);
                execution_tree::Expr::new_literal(literal)
            }
            syntax_tree::ExprInner::VarDef(variable_definition) => {
                let variable_definition =
                    self.parse_variable_definition(variable_definition, parser_scope);
                execution_tree::Expr::new_variable_definition(variable_definition)
            }
            syntax_tree::ExprInner::VarAssign(variable_assignment) => {
                let variable_assignment =
                    self.parse_variable_assignment(variable_assignment, parser_scope);
                execution_tree::Expr::new_variable_assignment(variable_assignment)
            }
            syntax_tree::ExprInner::VarCall(variable_call) => {
                let variable_call = self.parse_variable_call(variable_call, parser_scope);
                execution_tree::Expr::new_variable_call(variable_call)
            }
            syntax_tree::ExprInner::FnDef(function_definition) => {
                let function_definition =
                    self.parse_function_definition(function_definition, parser_scope);
                execution_tree::Expr::new_function_definition(function_definition)
            }
            syntax_tree::ExprInner::FnCall(function_call) => {
                let function_call = self.parse_function_call(function_call, parser_scope);
                execution_tree::Expr::new_function_call(function_call)
            }
            syntax_tree::ExprInner::FnRet(function_return) => {
                let function_return = self.parse_function_return(function_return, parser_scope);
                execution_tree::Expr::new_function_return(function_return)
            }
            syntax_tree::ExprInner::Loop(loop_) => {
                let loop_ = self.parse_loop(loop_, parser_scope);
                execution_tree::Expr::new_loop(loop_)
            }
            syntax_tree::ExprInner::LoopBr(loop_break) => {
                let loop_break = self.parse_loop_break(loop_break, parser_scope);
                execution_tree::Expr::new_loop_break(loop_break)
            }
            syntax_tree::ExprInner::Cond(condition) => {
                let condition = self.parse_condition(condition, parser_scope);
                execution_tree::Expr::new_condition(condition)
            }
        }
    }

    pub fn parse_literal(&mut self, literal: syntax_tree::Literal) -> execution_tree::Literal {
        let syntax_tree::Literal(value) = literal;
        execution_tree::Literal(value)
    }

    pub fn parse_variable_definition(
        &mut self,
        variable_definition: syntax_tree::VarDef,
        parser_scope: &ParserScope,
    ) -> execution_tree::VarDef {
        let syntax_tree::VarDef { name, value } = variable_definition;
        let value = self.parse_expression(value, parser_scope);
        let variable_id = parser_scope.add_name(name);
        execution_tree::VarDef { value, variable_id }
    }

    pub fn parse_variable_assignment(
        &mut self,
        variable_assignment: syntax_tree::VarAssign,
        parser_scope: &ParserScope,
    ) -> execution_tree::VarAssign {
        let syntax_tree::VarAssign { name, value } = variable_assignment;
        let value = self.parse_expression(value, parser_scope);
        let variable_id = parser_scope
            .get_variable_id(&name)
            .expect("assignment to undefined variable");
        execution_tree::VarAssign { value, variable_id }
    }

    pub fn parse_variable_call(
        &mut self,
        variable_call: syntax_tree::VarCall,
        parser_scope: &ParserScope,
    ) -> execution_tree::VarCall {
        let syntax_tree::VarCall { name } = variable_call;
        let variable_id = parser_scope
            .get_variable_id(&name)
            .expect("call of undefined variable");
        execution_tree::VarCall { variable_id }
    }

    pub fn parse_function_definition(
        &mut self,
        function_definition: syntax_tree::FnDef,
        parser_scope: &ParserScope,
    ) -> execution_tree::FnDef {
        let syntax_tree::FnDef {
            body,
            parameter_names,
        } = function_definition;

        let parser_scope = parser_scope.make_child_function();
        let parameter_ids = parameter_names
            .into_iter()
            .map(|name| parser_scope.add_name(name))
            .collect();
        let body_scope_id = self.parse_ast_scope(body, &parser_scope);

        execution_tree::FnDef {
            body_scope_id,
            parameter_ids,
        }
    }

    pub fn parse_function_call(
        &mut self,
        function_call: syntax_tree::FnCall,
        parser_scope: &ParserScope,
    ) -> execution_tree::FnCall {
        let syntax_tree::FnCall { name, arguments } = function_call;

        let variable_id = parser_scope
            .get_variable_id(&name)
            .expect(&format!("call of undeclared function '{name}'"));
        let parameters = arguments
            .into_iter()
            .map(|argument| self.parse_expression(argument, parser_scope))
            .collect();

        execution_tree::FnCall {
            arguments: parameters,
            variable_id,
        }
    }

    pub fn parse_function_return(
        &mut self,
        function_return: syntax_tree::FnRet,
        parser_scope: &ParserScope,
    ) -> execution_tree::FnRet {
        let syntax_tree::FnRet { value } = function_return;

        let value = self.parse_expression(value, parser_scope);
        let function_scope_id = parser_scope
            .get_current_function_id()
            .expect("returning outside a function");

        execution_tree::FnRet {
            value,
            function_scope_id,
        }
    }

    pub fn parse_loop(
        &mut self,
        loop_: syntax_tree::Loop,
        parser_scope: &ParserScope,
    ) -> execution_tree::Loop {
        let syntax_tree::Loop { body } = loop_;

        let parser_scope = parser_scope.make_child_loop();
        let body_scope_id = self.parse_ast_scope(body, &parser_scope);

        execution_tree::Loop { body_scope_id }
    }

    pub fn parse_loop_break(
        &mut self,
        loop_break: syntax_tree::LoopBr,
        parser_scope: &ParserScope,
    ) -> execution_tree::LoopBr {
        let syntax_tree::LoopBr { value } = loop_break;

        let value = self.parse_expression(value, parser_scope);
        let loop_scope_id = parser_scope
            .get_current_loop_id()
            .expect("breaking outside a loop");

        execution_tree::LoopBr {
            value,
            loop_scope_id,
        }
    }

    pub fn parse_condition(
        &mut self,
        condition: syntax_tree::Cond,
        parser_scope: &ParserScope,
    ) -> execution_tree::Cond {
        let syntax_tree::Cond {
            condition,
            arm_true,
            arm_false,
        } = condition;

        let condition = self.parse_expression(condition, parser_scope);
        let arm_true = self.parse_expression(arm_true, parser_scope);
        let arm_false = arm_false.map(|arm_false| self.parse_expression(arm_false, parser_scope));

        execution_tree::Cond {
            condition,
            arm_true,
            arm_false,
        }
    }
}

#[derive(Debug, Clone)]
struct ParserScopeVariables {
    parent_scope: Option<Rc<Mutex<ParserScopeVariables>>>,
    local_variables: HashMap<String, Id>,
}

impl ParserScopeVariables {
    pub fn get_id(&self, name: &str) -> Option<Id> {
        self.get_id_in_local(name)
            .or_else(|| self.get_id_in_parents(name))
    }

    fn get_id_in_local(&self, name: &str) -> Option<Id> {
        self.local_variables.get(name).cloned()
    }

    fn get_id_in_parents(&self, name: &str) -> Option<Id> {
        self.parent_scope
            .as_ref()
            .and_then(|parent| parent.lock().unwrap().get_id(name))
    }

    fn local_variable_ids(&self) -> Vec<Id> {
        self.local_variables.values().cloned().collect()
    }

    fn add_name(&mut self, name: String, id: Id) {
        let _dropped = self.local_variables.insert(name, id);
    }
}

/// Clonable, shareable.
#[derive(Debug, Clone)]
pub struct ParserScope {
    variables: Rc<Mutex<ParserScopeVariables>>,
    next_global_id: Rc<Mutex<Id>>,
    current_id: Id,
    parent_id: Option<Id>,
    current_function_scope_id: Option<Id>,
    current_loop_scope_id: Option<Id>,
}

impl ParserScope {
    pub fn new_root() -> Self {
        let current_id = Id::zero();
        let next_global_id = current_id.next();
        let variables = ParserScopeVariables {
            local_variables: HashMap::new(),
            parent_scope: None,
        };
        Self {
            parent_id: None,
            current_id,
            next_global_id: Rc::new(Mutex::new(next_global_id)),
            variables: Rc::new(Mutex::new(variables)),
            current_function_scope_id: None,
            current_loop_scope_id: None,
        }
    }

    pub fn make_child_common(&self) -> Self {
        let variables = ParserScopeVariables {
            local_variables: HashMap::new(),
            parent_scope: Some(self.variables.clone()),
        };
        Self {
            parent_id: Some(self.get_current_id()),
            next_global_id: self.next_global_id.clone(),
            variables: Rc::new(Mutex::new(variables)),
            current_id: self.request_new_id(),
            current_function_scope_id: self.get_current_function_id(),
            current_loop_scope_id: self.get_current_loop_id(),
        }
    }

    pub fn make_child_function(&self) -> Self {
        let variables = ParserScopeVariables {
            local_variables: HashMap::new(),
            parent_scope: Some(self.variables.clone()),
        };

        let current_id = self.request_new_id();

        Self {
            parent_id: Some(self.get_current_id()),
            next_global_id: self.next_global_id.clone(),
            variables: Rc::new(Mutex::new(variables)),
            current_id,
            current_function_scope_id: Some(current_id.clone()),
            current_loop_scope_id: None,
        }
    }

    pub fn make_child_loop(&self) -> Self {
        let variables = ParserScopeVariables {
            local_variables: HashMap::new(),
            parent_scope: Some(self.variables.clone()),
        };

        let current_id = self.request_new_id();

        Self {
            parent_id: Some(self.get_current_id()),
            next_global_id: self.next_global_id.clone(),
            variables: Rc::new(Mutex::new(variables)),
            current_id,
            current_function_scope_id: self.get_current_function_id(),
            current_loop_scope_id: Some(current_id.clone()),
        }
    }

    pub fn get_current_id(&self) -> Id {
        self.current_id.clone()
    }

    pub fn get_current_function_id(&self) -> Option<Id> {
        self.current_function_scope_id.clone()
    }

    pub fn get_current_loop_id(&self) -> Option<Id> {
        self.current_loop_scope_id.clone()
    }

    pub fn get_parent_id(&self) -> Option<Id> {
        self.parent_id.clone()
    }

    pub fn get_variable_id(&self, name: &str) -> Option<Id> {
        self.variables.lock().unwrap().get_id(name)
    }

    fn request_new_id(&self) -> Id {
        let mut next_id_ref = self.next_global_id.lock().unwrap();
        let id = *next_id_ref;
        *next_id_ref = id.next();
        id
    }

    pub fn add_name(&self, name: String) -> Id {
        let new_id = self.request_new_id();
        self.variables.lock().unwrap().add_name(name, new_id);
        new_id
    }

    pub fn add_anonymous(&self) -> Id {
        self.request_new_id()
    }

    pub fn local_variable_ids(&self) -> Vec<Id> {
        self.variables.lock().unwrap().local_variable_ids()
    }
}
