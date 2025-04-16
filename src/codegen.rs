//codegen is responsible for generating bytecode(Vector of Instruction) from AST

use std::collections::HashMap;

use crate::ast::{BinaryOp, Node, UnaryOp};
use crate::error::{CompilerError, Result};

/// Bytecode instructions for the virtual machine
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Stack operations
    PushInt(i64),
    PushFloat(f64),
    PushString(String),
    PushBool(bool),
    PushNull,
    Pop,

    // Variable operations
    LoadVar(String),
    StoreVar(String),

    // Array operations
    CreateArray,
    ArrayPush,
    ArraySet,
    ArrayGet,

    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,

    // Comparison operations
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical operations
    LogicalAnd,
    LogicalOr,
    LogicalNot,

    // Control flow
    Jump(usize),         // Jump to absolute address
    JumpIfFalse(usize),  // Jump to absolute address if top of stack is false
    JumpIfTrue(usize),   // Jump to absolute address if top of stack is true
    Label(usize),        // Label for jumps

    // Function operations
    Call(String, usize), // Function name, argument count
    Return,

    // I/O operations
    Echo,
    EchoLine, // Echo with a newline

    // String operations
    Concat,
}

/// Compiled function
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub param_count: usize,
    pub instructions: Vec<Instruction>,
}

/// Code generator for PHP AST
pub struct CodeGenerator {
    functions: HashMap<String, Function>,
    current_instructions: Vec<Instruction>,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            current_instructions: Vec::new(),
        }
    }

    /// Generate code for a node
    pub fn generate(&mut self, node: &Node) -> Result<Vec<Instruction>> {
        self.current_instructions.clear();
        self.generate_node(node)?;
        Ok(self.current_instructions.clone())
    }

    /// Get the compiled functions
    pub fn get_functions(&self) -> &HashMap<String, Function> {
        &self.functions
    }

    /// Generate code for a node
    fn generate_node(&mut self, node: &Node) -> Result<()> {
        match node {
            Node::Program(statements) => {
                for stmt in statements {
                    self.generate_node(stmt)?;
                }
            }
            Node::ExpressionStmt(expr) => {
                self.generate_node(expr)?;
                // Pop the result of the expression if it's not used
                self.current_instructions.push(Instruction::Pop);
            }
            Node::BlockStmt(statements, _) => {
                for stmt in statements {
                    self.generate_node(stmt)?;
                }
            }
            Node::IfStmt { condition, then_branch, else_branch, .. } => {
                // Generate code for the condition
                self.generate_node(condition)?;

                // Jump to else branch if condition is false
                let jump_to_else = self.current_instructions.len();
                self.current_instructions.push(Instruction::JumpIfFalse(0)); // Placeholder

                // Generate code for the then branch
                self.generate_node(then_branch)?;

                if let Some(else_branch) = else_branch {
                    // Jump over the else branch
                    let jump_over_else = self.current_instructions.len();
                    self.current_instructions.push(Instruction::Jump(0)); // Placeholder

                    // Update the jump to else
                    let else_start = self.current_instructions.len();
                    self.current_instructions[jump_to_else] = Instruction::JumpIfFalse(else_start);

                    // Add a label for the else branch
                    self.current_instructions.push(Instruction::Label(else_start));

                    // Generate code for the else branch
                    self.generate_node(else_branch)?;

                    // Update the jump over else
                    let after_else = self.current_instructions.len();
                    self.current_instructions[jump_over_else] = Instruction::Jump(after_else);

                    // Add a label for after the else branch
                    self.current_instructions.push(Instruction::Label(after_else));
                } else {
                    // Update the jump to else (which is actually the end of the if statement)
                    let after_if = self.current_instructions.len();
                    self.current_instructions[jump_to_else] = Instruction::JumpIfFalse(after_if);

                    // Add a label for after the if statement
                    self.current_instructions.push(Instruction::Label(after_if));
                }
            }
            Node::WhileStmt { condition, body, .. } => {
                // Loop start
                let loop_start = self.current_instructions.len();

                // Add a label for the loop start
                self.current_instructions.push(Instruction::Label(loop_start));

                // Generate code for the condition
                self.generate_node(condition)?;

                // Jump out of the loop if condition is false
                let jump_out = self.current_instructions.len();
                self.current_instructions.push(Instruction::JumpIfFalse(0)); // Placeholder

                // Generate code for the body
                self.generate_node(body)?;

                // Jump back to the start of the loop
                self.current_instructions.push(Instruction::Jump(loop_start));

                // Update the jump out
                let after_loop = self.current_instructions.len();
                self.current_instructions[jump_out] = Instruction::JumpIfFalse(after_loop);

                // Add a label for after the loop
                self.current_instructions.push(Instruction::Label(after_loop));
            }
            Node::ForStmt { init, condition, increment, body, .. } => {
                // Generate code for the initialization
                if let Some(init) = init {
                    self.generate_node(init)?;
                }

                // Loop start
                let loop_start = self.current_instructions.len();

                // Add a label for the loop start
                self.current_instructions.push(Instruction::Label(loop_start));

                // Generate code for the condition
                if let Some(condition) = condition {
                    self.generate_node(condition)?;
                } else {
                    // If no condition, use true
                    self.current_instructions.push(Instruction::PushBool(true));
                }

                // Jump out of the loop if condition is false
                let jump_out = self.current_instructions.len();
                self.current_instructions.push(Instruction::JumpIfFalse(0)); // Placeholder

                // Generate code for the body
                self.generate_node(body)?;

                // Generate code for the increment
                if let Some(increment) = increment {
                    self.generate_node(increment)?;
                    // Pop the result of the increment if it's not used
                    self.current_instructions.push(Instruction::Pop);
                }

                // Jump back to the start of the loop
                self.current_instructions.push(Instruction::Jump(loop_start));

                // Update the jump out
                let after_loop = self.current_instructions.len();
                self.current_instructions[jump_out] = Instruction::JumpIfFalse(after_loop);

                // Add a label for after the loop
                self.current_instructions.push(Instruction::Label(after_loop));
            }
            Node::ForeachStmt { array, .. } => {
                // This is a simplified implementation of foreach
                // In a real implementation, we would need to handle iterating over arrays

                // Generate code for the array
                self.generate_node(array)?;

                // TODO: Implement proper foreach loop
                // For now, we'll just generate a warning
                return Err(CompilerError::CodeGenError {
                    message: "Foreach loops are not fully implemented yet".to_string(),
                });
            }
            Node::ReturnStmt(value, _) => {
                if let Some(value) = value {
                    self.generate_node(value)?;
                } else {
                    // If no value, return null
                    self.current_instructions.push(Instruction::PushNull);
                }

                self.current_instructions.push(Instruction::Return);
            }
            Node::EchoStmt(expressions, _) => {
                // We need to check if this is a single expression or multiple expressions
                // If it's a single expression, we'll use EchoLine
                // If it's multiple expressions, we'll use Echo for all but the last one
                let expr_count = expressions.len();
                for (i, expr) in expressions.iter().enumerate() {
                    self.generate_node(expr)?;
                    if i == expr_count - 1 {
                        // Last expression, add a newline
                        self.current_instructions.push(Instruction::EchoLine);
                    } else {
                        // Not the last expression, don't add a newline
                        self.current_instructions.push(Instruction::Echo);
                    }
                }
            }
            Node::VarDecl { name, initializer, .. } => {
                if let Some(initializer) = initializer {
                    // Generate code for the initializer
                    self.generate_node(initializer)?;

                    // Store the value in the variable
                    self.current_instructions.push(Instruction::StoreVar(name.clone()));
                } else {
                    // If no initializer, use null
                    self.current_instructions.push(Instruction::PushNull);
                    self.current_instructions.push(Instruction::StoreVar(name.clone()));
                }
            }
            Node::FunctionDecl { name, params, body, .. } => {
                // Save the current instructions
                let saved_instructions = self.current_instructions.clone();
                self.current_instructions.clear();

                // Generate code for the function body
                self.generate_node(body)?;

                // Make sure the function returns
                if !self.current_instructions.iter().any(|i| matches!(i, Instruction::Return)) {
                    // If the function doesn't return, add a return null
                    self.current_instructions.push(Instruction::PushNull);
                    self.current_instructions.push(Instruction::Return);
                }

                // Create a new function
                let function = Function {
                    name: name.clone(),
                    param_count: params.len(),
                    instructions: self.current_instructions.clone(),
                };

                // Add the function to the map
                self.functions.insert(name.clone(), function);

                // Restore the current instructions
                self.current_instructions = saved_instructions;
            }
            Node::BinaryExpr { op, left, right, .. } => {
                match op {
                    BinaryOp::Assign => {
                        // For assignment, we need to get the variable name
                        if let Node::Variable(name, _) = &**left {
                            // Generate code for the right-hand side
                            self.generate_node(right)?;

                            // Store the value in the variable
                            self.current_instructions.push(Instruction::StoreVar(name.clone()));

                            // Load the variable again (assignment is an expression in PHP)
                            self.current_instructions.push(Instruction::LoadVar(name.clone()));
                        } else {
                            return Err(CompilerError::CodeGenError {
                                message: "Left-hand side of assignment must be a variable".to_string(),
                            });
                        }
                    }
                    _ => {
                        // Generate code for the left and right operands
                        self.generate_node(left)?;
                        self.generate_node(right)?;

                        // Generate code for the operation
                        match op {
                            BinaryOp::Add => self.current_instructions.push(Instruction::Add),
                            BinaryOp::Subtract => self.current_instructions.push(Instruction::Subtract),
                            BinaryOp::Multiply => self.current_instructions.push(Instruction::Multiply),
                            BinaryOp::Divide => self.current_instructions.push(Instruction::Divide),
                            BinaryOp::Modulo => self.current_instructions.push(Instruction::Modulo),
                            BinaryOp::Equal => self.current_instructions.push(Instruction::Equal),
                            BinaryOp::NotEqual => self.current_instructions.push(Instruction::NotEqual),
                            BinaryOp::Less => self.current_instructions.push(Instruction::Less),
                            BinaryOp::LessEqual => self.current_instructions.push(Instruction::LessEqual),
                            BinaryOp::Greater => self.current_instructions.push(Instruction::Greater),
                            BinaryOp::GreaterEqual => self.current_instructions.push(Instruction::GreaterEqual),
                            BinaryOp::LogicalAnd => self.current_instructions.push(Instruction::LogicalAnd),
                            BinaryOp::LogicalOr => self.current_instructions.push(Instruction::LogicalOr),
                            BinaryOp::Concat => self.current_instructions.push(Instruction::Concat),
                            BinaryOp::ArrayAccess => self.current_instructions.push(Instruction::ArrayGet),
                            BinaryOp::Assign => unreachable!(), // Handled above
                        }
                    }
                }
            }
            Node::UnaryExpr { op, expr, .. } => {
                // Generate code for the expression
                self.generate_node(expr)?;

                // Generate code for the operation
                match op {
                    UnaryOp::Negate => self.current_instructions.push(Instruction::Negate),
                    UnaryOp::LogicalNot => self.current_instructions.push(Instruction::LogicalNot),
                }
            }
            Node::Variable(name, _) => {
                // Load the variable
                self.current_instructions.push(Instruction::LoadVar(name.clone()));
            }
            Node::FunctionCall { name, args, .. } => {
                // Generate code for the arguments (in reverse order)
                for arg in args.iter().rev() {
                    self.generate_node(arg)?;
                }

                // Call the function
                self.current_instructions.push(Instruction::Call(name.clone(), args.len()));
            }
            Node::IntLiteral(value, _) => {
                self.current_instructions.push(Instruction::PushInt(*value));
            }
            Node::FloatLiteral(value, _) => {
                self.current_instructions.push(Instruction::PushFloat(*value));
            }
            Node::StringLiteral(value, _) => {
                self.current_instructions.push(Instruction::PushString(value.clone()));
            }
            Node::BooleanLiteral(value, _) => {
                self.current_instructions.push(Instruction::PushBool(*value));
            }
            Node::NullLiteral(_) => {
                self.current_instructions.push(Instruction::PushNull);
            }
            Node::ArrayLiteral(elements, _) => {
                // Create a new array
                self.current_instructions.push(Instruction::CreateArray);

                // Add each element to the array
                for (key, value) in elements {
                    // Generate code for the value
                    self.generate_node(value)?;

                    if let Some(key_node) = key {
                        // If a key is provided, generate code for it
                        self.generate_node(key_node)?;
                        // Set the element with the key
                        self.current_instructions.push(Instruction::ArraySet);
                    } else {
                        // If no key is provided, just push the value
                        self.current_instructions.push(Instruction::ArrayPush);
                    }
                }
            }
        }

        Ok(())
    }
}
