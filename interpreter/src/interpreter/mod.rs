use std::collections::HashMap;

use symboscript_types::{interpreter::*, lexer::*, parser::*};
use symboscript_utils::report_error;

pub struct Interpreter<'a> {
    /// Path of the source file
    path: &'a str,

    /// Source Text
    source: &'a str,

    ast: &'a Ast,

    scope_stack: Vec<String>,

    current_scope: String,

    vault: Vault,
}

impl<'a> Interpreter<'a> {
    pub fn new(path: &'a str, source: &'a str, ast: &'a Ast) -> Self {
        let vault = Vault::new();

        Self {
            path,
            source,
            ast,
            scope_stack: vec![],
            current_scope: String::new(),
            vault,
        }
    }

    pub fn run(&mut self) {
        self.initialize();

        self.eval_ast(self.ast.clone());
    }

    fn eval_ast(&mut self, ast: Ast) {
        self.eval_program_body(&ast.program.body);
    }

    fn eval_program_body(&mut self, body: &BlockStatement) {
        for statement in body {
            self.eval_statement(&statement);
        }
    }

    fn eval_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ExpressionStatement(expr) => {
                self.eval_expression(&expr);
            }
            Statement::ReturnStatement(_) => todo!(),
            Statement::ThrowStatement(_) => todo!(),
            Statement::ContinueStatement(_) => todo!(),
            Statement::BreakStatement(_) => todo!(),
            Statement::YieldStatement(_) => todo!(),
            Statement::VariableDeclaration(_) => todo!(),
            Statement::FunctionDeclaration(_) => todo!(),
            Statement::ScopeDeclaration(decl) => {
                self.enter_named_scope(&format!("{}", decl.id));
                self.eval_program_body(&decl.body);
                self.exit_named_scope();
            }
            Statement::IfStatement(_) => todo!(),
            Statement::ForStatement(_) => todo!(),
            Statement::WhileStatement(_) => todo!(),
            Statement::LoopStatement(_) => todo!(),
            Statement::TryStatement(_) => todo!(),
            Statement::BlockStatement(body) => {
                self.increment_scope();
                self.eval_program_body(body);
                self.decrement_scope();
            }
        }
    }

    fn eval_expression(&mut self, expression: &Expression) -> VariableValue {
        match expression {
            Expression::BinaryExpression(binary_expr) => self.eval_binary_expression(binary_expr),
            Expression::UnaryExpression(_) => todo!(),
            Expression::ConditionalExpression(_) => todo!(),
            Expression::CallExpression(_) => todo!(),
            Expression::MemberExpression(_) => todo!(),
            Expression::SequenceExpression(_) => todo!(),
            Expression::WordExpression(_) => todo!(),

            Expression::Literal(_) => return expression.clone(),

            Expression::Identifier(id) => return self.get_variable(id),

            Expression::None => return VariableValue::None,

            _ => {
                unreachable!("If you see this, something went wrong. Create an issue. https://github.com/symboscript/symboscript/issues/new")
            }
        }
    }

    fn eval_binary_expression(&mut self, expression: &BinaryExpression) -> Expression {
        let left = self.eval_expression(&expression.left);
        let right = self.eval_expression(&expression.right);

        match expression.operator {
            BinaryOperator::Plus => todo!(),
            BinaryOperator::Minus => todo!(),
            BinaryOperator::Multiply => todo!(),
            BinaryOperator::Divide => todo!(),
            BinaryOperator::Power => todo!(),
            BinaryOperator::Range => todo!(),

            BinaryOperator::Modulo => todo!(),

            BinaryOperator::And => todo!(),
            BinaryOperator::Or => todo!(),
            BinaryOperator::Xor => todo!(),

            BinaryOperator::BitAnd => todo!(),
            BinaryOperator::BitOr => todo!(),
            BinaryOperator::BitXor => todo!(),

            BinaryOperator::BitLeftShift => todo!(),
            BinaryOperator::BitRightShift => todo!(),

            BinaryOperator::Assign => todo!(),
            BinaryOperator::FormulaAssign => todo!(),
            BinaryOperator::PlusAssign => todo!(),
            BinaryOperator::MinusAssign => todo!(),
            BinaryOperator::MultiplyAssign => todo!(),
            BinaryOperator::DivideAssign => todo!(),
            BinaryOperator::PowerAssign => todo!(),
            BinaryOperator::ModuloAssign => todo!(),

            BinaryOperator::Equal => todo!(),
            BinaryOperator::NotEqual => todo!(),
            BinaryOperator::Less => todo!(),
            BinaryOperator::LessEqual => todo!(),
            BinaryOperator::Greater => todo!(),
            BinaryOperator::GreaterEqual => todo!(),
        }
    }

    fn get_variable(&mut self, identifier: &String) -> ScopeValues {
        let (scope_name, _) = self.parse_current_scope();
    }

    fn initialize(&mut self) {
        self.vault.insert("std.$0".to_owned(), ScopeValue::new());
        self.scope_stack.push("std.$0".to_owned());
        self.update_current_scope();
        self.add_native_functions();

        self.vault.insert("global.$0".to_owned(), ScopeValue::new());
        self.scope_stack.push("global.$0".to_owned());
        self.update_current_scope();
    }

    fn add_native_functions(&mut self) {
        let scope = self.get_curr_value();

        scope.insert(
            "print".to_owned(),
            ScopeValues::NativeFunction(NativeFunction::Print),
        );

        scope.insert(
            "println".to_owned(),
            ScopeValues::NativeFunction(NativeFunction::Println),
        );
    }

    /// Initializes a new named scope
    fn enter_named_scope(&mut self, name: &str) {
        let (scope_name, _) = self.parse_current_scope();

        let new_scope = format!("{}.{}.$0", scope_name, name);

        self.send_scope_ref(&new_scope);

        self.init_scope(new_scope);
    }

    /// Exits the current named scope
    fn exit_named_scope(&mut self) {
        // named scopes not clears when exiting
        // named scopes cleared only when decrementing scope
        self.scope_stack.pop();
        self.update_current_scope();
    }

    /// Adds a reference to the current scope
    fn send_scope_ref(&mut self, name: &str) {
        self.get_curr_scope_refs().push(name.to_owned());
    }

    /// Increments the current scope
    fn increment_scope(&mut self) {
        let (scope_name, num) = self.parse_current_scope();

        let new_scope = format!("{}.${}", scope_name, num + 1);

        self.init_scope(new_scope);
    }

    /// Decrements the current scope and deletes named scopes in the current scope
    fn decrement_scope(&mut self) {
        let scope = self.current_scope.clone();

        for ref_name in self.get_curr_scope_refs().clone() {
            self.vault.remove(&ref_name);
        }

        self.vault.remove(&scope);
        self.scope_stack.pop();

        self.update_current_scope();
    }

    /// Initializes the current scope
    fn init_scope(&mut self, scope_name: String) {
        self.vault.insert(scope_name.clone(), ScopeValue::new());
        self.scope_stack.push(scope_name);
        self.update_current_scope();
    }

    /// Parses the current scope name and number
    fn parse_current_scope(&mut self) -> (String, usize) {
        let (scope_name, num) = self.current_scope.rsplit_once(".$").unwrap();
        let num = num.parse::<usize>().unwrap();

        (scope_name.to_owned(), num)
    }

    /// Gets the current scope values
    fn get_curr_value(&mut self) -> &mut HashMap<String, ScopeValues> {
        &mut self
            .vault
            .get_mut(self.current_scope.as_str())
            .unwrap()
            .values
    }

    /// Gets the current named scopes in the current scope
    fn get_curr_scope_refs(&mut self) -> &mut Vec<String> {
        &mut self
            .vault
            .get_mut(self.current_scope.as_str())
            .unwrap()
            .named_scope_refs
    }

    /// Updates the current scope
    fn update_current_scope(&mut self) {
        self.current_scope = self.scope_stack.last().unwrap().clone();
    }

    /// Reports an interpreter error
    fn report(&self, error: &str, start: usize, end: usize) {
        report_error(self.path, self.source, error, start, end);
    }
}