use std::collections::HashMap;

use crate::ast::{BinaryOp, Location, Node, Type, UnaryOp};
use crate::error::{CompilerError, type_error, Result};

/// Type checker for PHP code
pub struct TypeChecker {
    variables: HashMap<String, Type>,
    functions: HashMap<String, (Vec<Type>, Type)>, // (param_types, return_type)
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        // Add built-in functions
        functions.insert("strlen".to_string(), (vec![Type::String], Type::Integer));
        functions.insert("substr".to_string(), (vec![Type::String, Type::Integer, Type::Integer], Type::String));

        Self {
            variables: HashMap::new(),
            functions,
        }
    }

    /// Check a program
    pub fn check_program(&mut self, node: &Node) -> Result<Type> {
        match node {
            Node::Program(statements) => {
                for stmt in statements {
                    self.check_node(stmt)?;
                }
                Ok(Type::Null)
            }
            _ => Err(type_error(
                &self.get_location(node),
                "Expected program",
            )),
        }
    }

    /// Check a node
    fn check_node(&mut self, node: &Node) -> Result<Type> {
        match node {
            Node::Program(_) => self.check_program(node),
            Node::ExpressionStmt(expr) => self.check_node(expr),
            Node::BlockStmt(statements, _) => {
                // Create a new scope
                let old_variables = self.variables.clone();

                // Check all statements
                for stmt in statements {
                    self.check_node(stmt)?;
                }

                // Restore the old scope
                self.variables = old_variables;

                Ok(Type::Null)
            }
            Node::IfStmt { condition, then_branch, else_branch, .. } => {
                // Check condition
                let cond_type = self.check_node(condition)?;

                // PHP is loosely typed, so we don't need to check if condition is boolean

                // Check branches
                self.check_node(then_branch)?;

                if let Some(else_branch) = else_branch {
                    self.check_node(else_branch)?;
                }

                Ok(Type::Null)
            }
            Node::WhileStmt { condition, body, .. } => {
                // Check condition
                let cond_type = self.check_node(condition)?;

                // PHP is loosely typed, so we don't need to check if condition is boolean

                // Check body
                self.check_node(body)?;

                Ok(Type::Null)
            }
            Node::ForStmt { init, condition, increment, body, .. } => {
                // Check initializer
                if let Some(init) = init {
                    self.check_node(init)?;
                }

                // Check condition
                if let Some(condition) = condition {
                    let cond_type = self.check_node(condition)?;
                    // PHP is loosely typed, so we don't need to check if condition is boolean
                }

                // Check increment
                if let Some(increment) = increment {
                    self.check_node(increment)?;
                }

                // Check body
                self.check_node(body)?;

                Ok(Type::Null)
            }
            Node::ForeachStmt { array, value_var, key_var, body, .. } => {
                // Check array
                let array_type = self.check_node(array)?;

                // PHP is loosely typed, so we don't need to check if array is actually an array

                // Add value variable to scope
                self.variables.insert(value_var.clone(), Type::Mixed);

                // Add key variable to scope if present
                if let Some(key_var) = key_var {
                    self.variables.insert(key_var.clone(), Type::Mixed);
                }

                // Check body
                self.check_node(body)?;

                Ok(Type::Null)
            }
            Node::ReturnStmt(value, _) => {
                if let Some(value) = value {
                    self.check_node(value)
                } else {
                    Ok(Type::Null)
                }
            }
            Node::EchoStmt(expressions, _) => {
                for expr in expressions {
                    self.check_node(expr)?;
                    // PHP can echo any type
                }

                Ok(Type::Null)
            }
            Node::VarDecl { name, initializer, .. } => {
                let var_type = if let Some(initializer) = initializer {
                    self.check_node(initializer)?
                } else {
                    Type::Null
                };

                self.variables.insert(name.clone(), var_type.clone());

                Ok(var_type)
            }
            Node::FunctionDecl { name, params, body, .. } => {
                // Create a new scope
                let old_variables = self.variables.clone();

                // Add parameters to scope
                let mut param_types = Vec::new();
                for (param_name, param_type) in params {
                    let type_ = param_type.clone().unwrap_or(Type::Mixed);
                    self.variables.insert(param_name.clone(), type_.clone());
                    param_types.push(type_);
                }

                // Check body
                self.check_node(body)?;

                // Add function to scope
                self.functions.insert(name.clone(), (param_types, Type::Mixed));

                // Restore the old scope
                self.variables = old_variables;

                Ok(Type::Null)
            }
            Node::BinaryExpr { op, left, right, .. } => {
                let left_type = self.check_node(left)?;
                let right_type = self.check_node(right)?;

                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                        // PHP is loosely typed, so we'll just return a numeric type
                        if left_type == Type::Float || right_type == Type::Float {
                            Ok(Type::Float)
                        } else {
                            Ok(Type::Integer)
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                        // Comparison operators return boolean
                        Ok(Type::Boolean)
                    }
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                        // Logical operators return boolean
                        Ok(Type::Boolean)
                    }
                    BinaryOp::Assign => {
                        // Assignment returns the assigned value
                        self.variables.insert(self.get_variable_name(left)?, right_type.clone());
                        Ok(right_type)
                    }
                    BinaryOp::Concat => {
                        // String concatenation returns string
                        Ok(Type::String)
                    }
                    BinaryOp::ArrayAccess => {
                        // Array access returns the element type, which is Mixed in PHP
                        if left_type == Type::Array {
                            Ok(Type::Mixed)
                        } else {
                            // Get the location from the left node
                            let location = match left.as_ref() {
                                Node::Variable(_, loc) => loc.clone(),
                                _ => Location { file: "unknown".to_string(), line: 0, column: 0 },
                            };
                            Err(CompilerError::TypeError(
                                location,
                                format!("Cannot access non-array type {:?} as array", left_type),
                            ))
                        }
                    }
                }
            }
            Node::UnaryExpr { op, expr, .. } => {
                let expr_type = self.check_node(expr)?;

                match op {
                    UnaryOp::Negate => {
                        // Negation returns a numeric type
                        if expr_type == Type::Float {
                            Ok(Type::Float)
                        } else {
                            Ok(Type::Integer)
                        }
                    }
                    UnaryOp::LogicalNot => {
                        // Logical not returns boolean
                        Ok(Type::Boolean)
                    }
                }
            }
            Node::Variable(name, _) => {
                // Look up variable in scope
                if let Some(type_) = self.variables.get(name) {
                    Ok(type_.clone())
                } else {
                    // In PHP, using an undefined variable is allowed (it's treated as null)
                    self.variables.insert(name.clone(), Type::Null);
                    Ok(Type::Null)
                }
            }
            Node::FunctionCall { name, args, location } => {
                // Check arguments
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.check_node(arg)?);
                }

                // Look up function in scope
                if let Some((param_types, return_type)) = self.functions.get(name) {
                    // PHP is loosely typed, so we don't need to check if argument types match parameter types
                    // But we could add some basic checks here

                    Ok(return_type.clone())
                } else {
                    // In PHP, calling an undefined function is an error
                    Err(type_error(
                        location,
                        format!("Undefined function: {}", name),
                    ))
                }
            }
            Node::IntLiteral(_, _) => Ok(Type::Integer),
            Node::FloatLiteral(_, _) => Ok(Type::Float),
            Node::StringLiteral(_, _) => Ok(Type::String),
            Node::BooleanLiteral(_, _) => Ok(Type::Boolean),
            Node::NullLiteral(_) => Ok(Type::Null),
            Node::ArrayLiteral(_, _) => Ok(Type::Array),
        }
    }

    /// Get the location of a node
    fn get_location(&self, node: &Node) -> crate::ast::Location {
        match node {
            Node::Program(_) => crate::ast::Location {
                file: "unknown".to_string(),
                line: 0,
                column: 0,
            },
            Node::ExpressionStmt(expr) => self.get_location(expr),
            Node::BlockStmt(_, location) => location.clone(),
            Node::IfStmt { location, .. } => location.clone(),
            Node::WhileStmt { location, .. } => location.clone(),
            Node::ForStmt { location, .. } => location.clone(),
            Node::ForeachStmt { location, .. } => location.clone(),
            Node::ReturnStmt(_, location) => location.clone(),
            Node::EchoStmt(_, location) => location.clone(),
            Node::VarDecl { location, .. } => location.clone(),
            Node::FunctionDecl { location, .. } => location.clone(),
            Node::BinaryExpr { location, .. } => location.clone(),
            Node::UnaryExpr { location, .. } => location.clone(),
            Node::Variable(_, location) => location.clone(),
            Node::FunctionCall { location, .. } => location.clone(),
            Node::IntLiteral(_, location) => location.clone(),
            Node::FloatLiteral(_, location) => location.clone(),
            Node::StringLiteral(_, location) => location.clone(),
            Node::BooleanLiteral(_, location) => location.clone(),
            Node::NullLiteral(location) => location.clone(),
            Node::ArrayLiteral(_, location) => location.clone(),
        }
    }

    /// Get the name of a variable from a node
    fn get_variable_name(&self, node: &Node) -> Result<String> {
        match node {
            Node::Variable(name, _) => Ok(name.clone()),
            _ => Err(type_error(
                &self.get_location(node),
                "Expected variable",
            )),
        }
    }
}
