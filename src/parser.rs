use std::iter::Peekable;
use std::slice::Iter;

use crate::ast::{BinaryOp, Location, Node, UnaryOp};
use crate::error::{syntax_error, Result};
use crate::lexer::{Token, TokenKind};

/// Parser for PHP source code
pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    current: Option<&'a Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let mut iter = tokens.iter().peekable();
        let current = iter.next();

        Self {
            tokens: iter,
            current,
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    /// Peek at the next token without advancing
    fn peek(&mut self) -> Option<&'a Token> {
        self.tokens.peek().copied()
    }

    /// Check if the current token matches the expected kind
    fn check(&self, kind: &TokenKind) -> bool {
        match self.current {
            Some(token) => std::mem::discriminant(&token.kind) == std::mem::discriminant(kind),
            None => false,
        }
    }

    /// Check if the current token is a specific kind with a specific value
    fn check_specific(&self, kind: &TokenKind) -> bool {
        match self.current {
            Some(token) => &token.kind == kind,
            None => false,
        }
    }

    /// Consume the current token if it matches the expected kind
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume the current token if it matches the expected kind, otherwise return an error
    fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<&'a Token> {
        match self.current {
            Some(token) if std::mem::discriminant(&token.kind) == std::mem::discriminant(kind) => {
                let current = token;
                self.advance();
                Ok(current)
            }
            Some(token) => Err(syntax_error(
                &token.location,
                format!("{}, found {:?}", message, token.kind),
            )),
            None => Err(syntax_error(
                &Location {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                format!("{}, found end of file", message),
            )),
        }
    }

    /// Parse a program
    pub fn parse_program(&mut self) -> Result<Node> {
        // Skip PHP open tag if present
        if self.check(&TokenKind::PhpOpen) {
            self.advance();
        }

        let mut statements = Vec::new();

        while self.current.is_some() && !self.check(&TokenKind::Eof) && !self.check(&TokenKind::PhpClose) {
            statements.push(self.parse_statement()?);
        }

        Ok(Node::Program(statements))
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Node> {
        match self.current {
            Some(token) => match &token.kind {
                TokenKind::Echo => self.parse_echo_statement(),
                TokenKind::If => self.parse_if_statement(),
                TokenKind::While => self.parse_while_statement(),
                TokenKind::For => self.parse_for_statement(),
                TokenKind::Foreach => self.parse_foreach_statement(),
                TokenKind::Function => self.parse_function_declaration(),
                TokenKind::Return => self.parse_return_statement(),
                TokenKind::LeftBrace => self.parse_block(),
                TokenKind::Variable(_) => {
                    // Variable assignment or expression
                    let expr = self.parse_expression()?;
                    self.expect(&TokenKind::Semicolon, "Expected ';' after expression")?;
                    Ok(Node::ExpressionStmt(Box::new(expr)))
                }
                _ => {
                    // Other expressions
                    let expr = self.parse_expression()?;
                    self.expect(&TokenKind::Semicolon, "Expected ';' after expression")?;
                    Ok(Node::ExpressionStmt(Box::new(expr)))
                }
            },
            None => Err(syntax_error(
                &Location {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                "Unexpected end of file",
            )),
        }
    }

    /// Parse an echo statement
    fn parse_echo_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'echo'

        let mut expressions = Vec::new();

        // Parse at least one expression
        expressions.push(self.parse_expression()?);

        // Parse additional expressions separated by commas
        while self.match_token(&TokenKind::Comma) {
            expressions.push(self.parse_expression()?);
        }

        self.expect(&TokenKind::Semicolon, "Expected ';' after echo statement")?;

        Ok(Node::EchoStmt(expressions, location))
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'if'

        self.expect(&TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::RightParen, "Expected ')' after condition")?;

        let then_branch = self.parse_statement()?;

        let else_branch = if self.match_token(&TokenKind::Else) {
            Some(Box::new(self.parse_statement()?))
        } else if self.match_token(&TokenKind::ElseIf) {
            // elseif is equivalent to else { if ... }
            Some(Box::new(self.parse_if_statement()?))
        } else {
            None
        };

        Ok(Node::IfStmt {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            location,
        })
    }

    /// Parse a while statement
    fn parse_while_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'while'

        self.expect(&TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::RightParen, "Expected ')' after condition")?;

        let body = self.parse_statement()?;

        Ok(Node::WhileStmt {
            condition: Box::new(condition),
            body: Box::new(body),
            location,
        })
    }

    /// Parse a for statement
    fn parse_for_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'for'

        self.expect(&TokenKind::LeftParen, "Expected '(' after 'for'")?;

        // Parse initializer
        let init = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            let init_expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon, "Expected ';' after for initializer")?;
            Some(Box::new(init_expr))
        };

        // Parse condition
        let condition = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            let cond_expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon, "Expected ';' after for condition")?;
            Some(Box::new(cond_expr))
        };

        // Parse increment
        let increment = if self.match_token(&TokenKind::RightParen) {
            None
        } else {
            let inc_expr = self.parse_expression()?;
            self.expect(&TokenKind::RightParen, "Expected ')' after for increment")?;
            Some(Box::new(inc_expr))
        };

        let body = self.parse_statement()?;

        Ok(Node::ForStmt {
            init,
            condition,
            increment,
            body: Box::new(body),
            location,
        })
    }

    /// Parse a foreach statement
    fn parse_foreach_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'foreach'

        self.expect(&TokenKind::LeftParen, "Expected '(' after 'foreach'")?;

        // Parse array expression
        let array = self.parse_expression()?;

        self.expect(&TokenKind::As, "Expected 'as' after array expression in foreach")?;

        // Parse key => value or just value
        let (key_var, value_var) = if let Some(token) = self.current {
            if let TokenKind::Variable(name) = &token.kind {
                let key_name = name.clone();
                self.advance();

                if self.match_token(&TokenKind::DoubleArrow) {
                    // key => value syntax
                    if let Some(value_token) = self.current {
                        if let TokenKind::Variable(value_name) = &value_token.kind {
                            self.advance();
                            (Some(key_name), value_name.clone())
                        } else {
                            return Err(syntax_error(
                                &value_token.location,
                                "Expected variable after '=>' in foreach",
                            ));
                        }
                    } else {
                        return Err(syntax_error(
                            &location,
                            "Unexpected end of file in foreach",
                        ));
                    }
                } else {
                    // Just value syntax
                    (None, key_name)
                }
            } else {
                return Err(syntax_error(
                    &token.location,
                    "Expected variable after 'as' in foreach",
                ));
            }
        } else {
            return Err(syntax_error(
                &location,
                "Unexpected end of file in foreach",
            ));
        };

        self.expect(&TokenKind::RightParen, "Expected ')' after foreach parameters")?;

        let body = self.parse_statement()?;

        Ok(Node::ForeachStmt {
            array: Box::new(array),
            value_var,
            key_var,
            body: Box::new(body),
            location,
        })
    }

    /// Parse a function declaration
    fn parse_function_declaration(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'function'

        // Parse function name
        let name = if let Some(token) = self.current {
            if let TokenKind::Identifier(name) = &token.kind {
                self.advance();
                name.clone()
            } else {
                return Err(syntax_error(
                    &token.location,
                    "Expected function name",
                ));
            }
        } else {
            return Err(syntax_error(
                &location,
                "Unexpected end of file",
            ));
        };

        self.expect(&TokenKind::LeftParen, "Expected '(' after function name")?;

        // Parse parameters
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                if let Some(token) = self.current {
                    if let TokenKind::Variable(name) = &token.kind {
                        let param_name = name.clone();
                        self.advance();

                        // PHP is dynamically typed, so we don't have explicit types
                        // But we could add type hints later
                        params.push((param_name, None));
                    } else {
                        return Err(syntax_error(
                            &token.location,
                            "Expected parameter name",
                        ));
                    }
                } else {
                    return Err(syntax_error(
                        &location,
                        "Unexpected end of file",
                    ));
                }

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse function body
        let body = self.parse_block()?;

        Ok(Node::FunctionDecl {
            name,
            params,
            body: Box::new(body),
            location,
        })
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.advance(); // Skip 'return'

        let value = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            let expr = self.parse_expression()?;
            self.expect(&TokenKind::Semicolon, "Expected ';' after return value")?;
            Some(Box::new(expr))
        };

        Ok(Node::ReturnStmt(value, location))
    }

    /// Parse a block statement
    fn parse_block(&mut self) -> Result<Node> {
        let location = self.current.unwrap().location.clone();
        self.expect(&TokenKind::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && self.current.is_some() {
            statements.push(self.parse_statement()?);
        }

        self.expect(&TokenKind::RightBrace, "Expected '}'")?;

        Ok(Node::BlockStmt(statements, location))
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Node> {
        self.parse_assignment()
    }

    /// Parse an assignment expression
    fn parse_assignment(&mut self) -> Result<Node> {
        let expr = self.parse_logical_or()?;

        if self.match_token(&TokenKind::Assign) {
            let location = self.current.unwrap().location.clone();
            let value = self.parse_assignment()?;

            // Check that the left side is a valid assignment target
            match expr {
                Node::Variable(_, _) => {
                    Ok(Node::BinaryExpr {
                        op: BinaryOp::Assign,
                        left: Box::new(expr),
                        right: Box::new(value),
                        location,
                    })
                }
                _ => Err(syntax_error(
                    &location,
                    "Invalid assignment target",
                )),
            }
        } else if self.match_token(&TokenKind::PlusAssign) {
            let location = self.current.unwrap().location.clone();
            let value = self.parse_assignment()?;

            // a += b is equivalent to a = a + b
            match expr {
                Node::Variable(ref name, ref var_loc) => {
                    let var_expr = Node::Variable(name.clone(), var_loc.clone());
                    let add_expr = Node::BinaryExpr {
                        op: BinaryOp::Add,
                        left: Box::new(var_expr),
                        right: Box::new(value),
                        location: location.clone(),
                    };

                    Ok(Node::BinaryExpr {
                        op: BinaryOp::Assign,
                        left: Box::new(expr),
                        right: Box::new(add_expr),
                        location,
                    })
                }
                _ => Err(syntax_error(
                    &location,
                    "Invalid assignment target",
                )),
            }
        } else if self.match_token(&TokenKind::MinusAssign) {
            let location = self.current.unwrap().location.clone();
            let value = self.parse_assignment()?;

            // a -= b is equivalent to a = a - b
            match expr {
                Node::Variable(ref name, ref var_loc) => {
                    let var_expr = Node::Variable(name.clone(), var_loc.clone());
                    let sub_expr = Node::BinaryExpr {
                        op: BinaryOp::Subtract,
                        left: Box::new(var_expr),
                        right: Box::new(value),
                        location: location.clone(),
                    };

                    Ok(Node::BinaryExpr {
                        op: BinaryOp::Assign,
                        left: Box::new(expr),
                        right: Box::new(sub_expr),
                        location,
                    })
                }
                _ => Err(syntax_error(
                    &location,
                    "Invalid assignment target",
                )),
            }
        } else if self.match_token(&TokenKind::MultiplyAssign) {
            let location = self.current.unwrap().location.clone();
            let value = self.parse_assignment()?;

            // a *= b is equivalent to a = a * b
            match expr {
                Node::Variable(ref name, ref var_loc) => {
                    let var_expr = Node::Variable(name.clone(), var_loc.clone());
                    let mul_expr = Node::BinaryExpr {
                        op: BinaryOp::Multiply,
                        left: Box::new(var_expr),
                        right: Box::new(value),
                        location: location.clone(),
                    };

                    Ok(Node::BinaryExpr {
                        op: BinaryOp::Assign,
                        left: Box::new(expr),
                        right: Box::new(mul_expr),
                        location,
                    })
                }
                _ => Err(syntax_error(
                    &location,
                    "Invalid assignment target",
                )),
            }
        } else if self.match_token(&TokenKind::DivideAssign) {
            let location = self.current.unwrap().location.clone();
            let value = self.parse_assignment()?;

            // a /= b is equivalent to a = a / b
            match expr {
                Node::Variable(ref name, ref var_loc) => {
                    let var_expr = Node::Variable(name.clone(), var_loc.clone());
                    let div_expr = Node::BinaryExpr {
                        op: BinaryOp::Divide,
                        left: Box::new(var_expr),
                        right: Box::new(value),
                        location: location.clone(),
                    };

                    Ok(Node::BinaryExpr {
                        op: BinaryOp::Assign,
                        left: Box::new(expr),
                        right: Box::new(div_expr),
                        location,
                    })
                }
                _ => Err(syntax_error(
                    &location,
                    "Invalid assignment target",
                )),
            }
        } else if self.match_token(&TokenKind::ConcatAssign) {
            let location = self.current.unwrap().location.clone();
            let value = self.parse_assignment()?;

            // a .= b is equivalent to a = a . b
            match expr {
                Node::Variable(ref name, ref var_loc) => {
                    let var_expr = Node::Variable(name.clone(), var_loc.clone());
                    let concat_expr = Node::BinaryExpr {
                        op: BinaryOp::Concat,
                        left: Box::new(var_expr),
                        right: Box::new(value),
                        location: location.clone(),
                    };

                    Ok(Node::BinaryExpr {
                        op: BinaryOp::Assign,
                        left: Box::new(expr),
                        right: Box::new(concat_expr),
                        location,
                    })
                }
                _ => Err(syntax_error(
                    &location,
                    "Invalid assignment target",
                )),
            }
        } else {
            Ok(expr)
        }
    }

    /// Parse a logical OR expression
    fn parse_logical_or(&mut self) -> Result<Node> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenKind::LogicalOr) || self.check_specific(&TokenKind::Or) {
            if self.check_specific(&TokenKind::Or) {
                self.advance(); // Skip 'or'
            }

            let location = self.current.unwrap().location.clone();
            let right = self.parse_logical_and()?;

            expr = Node::BinaryExpr {
                op: BinaryOp::LogicalOr,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a logical AND expression
    fn parse_logical_and(&mut self) -> Result<Node> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::LogicalAnd) || self.check_specific(&TokenKind::And) {
            if self.check_specific(&TokenKind::And) {
                self.advance(); // Skip 'and'
            }

            let location = self.current.unwrap().location.clone();
            let right = self.parse_equality()?;

            expr = Node::BinaryExpr {
                op: BinaryOp::LogicalAnd,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse an equality expression
    fn parse_equality(&mut self) -> Result<Node> {
        let mut expr = self.parse_relational()?;

        loop {
            let op = if self.match_token(&TokenKind::Equal) {
                BinaryOp::Equal
            } else if self.match_token(&TokenKind::NotEqual) {
                BinaryOp::NotEqual
            } else if self.match_token(&TokenKind::Identical) {
                // For simplicity, we'll treat === the same as == for now
                BinaryOp::Equal
            } else if self.match_token(&TokenKind::NotIdentical) {
                // For simplicity, we'll treat !== the same as != for now
                BinaryOp::NotEqual
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_relational()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a relational expression
    fn parse_relational(&mut self) -> Result<Node> {
        let mut expr = self.parse_additive()?;

        loop {
            let op = if self.match_token(&TokenKind::LessThan) {
                BinaryOp::Less
            } else if self.match_token(&TokenKind::LessThanEqual) {
                BinaryOp::LessEqual
            } else if self.match_token(&TokenKind::GreaterThan) {
                BinaryOp::Greater
            } else if self.match_token(&TokenKind::GreaterThanEqual) {
                BinaryOp::GreaterEqual
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_additive()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse an additive expression
    fn parse_additive(&mut self) -> Result<Node> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = if self.match_token(&TokenKind::Plus) {
                BinaryOp::Add
            } else if self.match_token(&TokenKind::Minus) {
                BinaryOp::Subtract
            } else if self.match_token(&TokenKind::Concat) {
                BinaryOp::Concat
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_multiplicative()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a multiplicative expression
    fn parse_multiplicative(&mut self) -> Result<Node> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = if self.match_token(&TokenKind::Asterisk) {
                BinaryOp::Multiply
            } else if self.match_token(&TokenKind::Slash) {
                BinaryOp::Divide
            } else if self.match_token(&TokenKind::Percent) {
                BinaryOp::Modulo
            } else {
                break;
            };

            let location = self.current.unwrap().location.clone();
            let right = self.parse_unary()?;

            expr = Node::BinaryExpr {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a unary expression
    fn parse_unary(&mut self) -> Result<Node> {
        if let Some(token) = self.current {
            let op = match token.kind {
                TokenKind::Minus => {
                    self.advance();
                    Some(UnaryOp::Negate)
                }
                TokenKind::LogicalNot => {
                    self.advance();
                    Some(UnaryOp::LogicalNot)
                }
                _ => None,
            };

            if let Some(op) = op {
                let location = token.location.clone();
                let expr = self.parse_unary()?;

                return Ok(Node::UnaryExpr {
                    op,
                    expr: Box::new(expr),
                    location,
                });
            }
        }

        self.parse_primary()
    }

    /// Parse a primary expression
    fn parse_primary(&mut self) -> Result<Node> {
        let expr = self.parse_primary_inner()?;
        self.parse_array_access(expr)
    }

    /// Parse array access after a primary expression
    fn parse_array_access(&mut self, mut expr: Node) -> Result<Node> {
        // Check for array access: expr[index]
        while self.match_token(&TokenKind::LeftBracket) {
            let location = self.current.unwrap().location.clone();
            let index = self.parse_expression()?;
            self.expect(&TokenKind::RightBracket, "Expected ']' after array index")?;

            expr = Node::BinaryExpr {
                op: BinaryOp::ArrayAccess,
                left: Box::new(expr),
                right: Box::new(index),
                location,
            };
        }

        Ok(expr)
    }

    /// Parse a primary expression (inner implementation)
    fn parse_primary_inner(&mut self) -> Result<Node> {
        match self.current {
            Some(token) => {
                let location = token.location.clone();

                match &token.kind {
                    TokenKind::IntLiteral(value) => {
                        self.advance();
                        Ok(Node::IntLiteral(*value, location))
                    }
                    TokenKind::FloatLiteral(value) => {
                        self.advance();
                        Ok(Node::FloatLiteral(*value, location))
                    }
                    TokenKind::StringLiteral(value) => {
                        self.advance();
                        Ok(Node::StringLiteral(value.clone(), location))
                    }
                    TokenKind::True => {
                        self.advance();
                        Ok(Node::BooleanLiteral(true, location))
                    }
                    TokenKind::False => {
                        self.advance();
                        Ok(Node::BooleanLiteral(false, location))
                    }
                    TokenKind::Null => {
                        self.advance();
                        Ok(Node::NullLiteral(location))
                    }
                    TokenKind::Variable(name) => {
                        self.advance();
                        Ok(Node::Variable(name.clone(), location))
                    }
                    TokenKind::LeftParen => {
                        self.advance();
                        let expr = self.parse_expression()?;
                        self.expect(&TokenKind::RightParen, "Expected ')' after expression")?;
                        Ok(expr)
                    }
                    TokenKind::LeftBracket => {
                        // Array literal
                        self.advance();

                        let mut elements = Vec::new();

                        if !self.check(&TokenKind::RightBracket) {
                            loop {
                                // Parse key => value or just value
                                let key = if self.peek().map_or(false, |t|
                                    matches!(t.kind, TokenKind::DoubleArrow)) {
                                    // Key is present
                                    let key_expr = self.parse_expression()?;
                                    self.expect(&TokenKind::DoubleArrow, "Expected '=>' after array key")?;
                                    Some(key_expr)
                                } else {
                                    None
                                };

                                let value = self.parse_expression()?;
                                elements.push((key, value));

                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }

                                // Allow trailing comma
                                if self.check(&TokenKind::RightBracket) {
                                    break;
                                }
                            }
                        }

                        self.expect(&TokenKind::RightBracket, "Expected ']' after array elements")?;

                        Ok(Node::ArrayLiteral(elements, location))
                    }
                    TokenKind::Identifier(name) => {
                        self.advance();

                        // Check if it's a function call
                        if self.check(&TokenKind::LeftParen) {
                            self.advance(); // Skip (

                            let mut args = Vec::new();

                            if !self.check(&TokenKind::RightParen) {
                                loop {
                                    args.push(self.parse_expression()?);

                                    if !self.match_token(&TokenKind::Comma) {
                                        break;
                                    }
                                }
                            }

                            self.expect(&TokenKind::RightParen, "Expected ')' after arguments")?;

                            Ok(Node::FunctionCall {
                                name: name.clone(),
                                args,
                                location,
                            })
                        } else {
                            // Treat as a constant (PHP constants don't have $ prefix)
                            // For simplicity, we'll treat constants as variables
                            Ok(Node::Variable(name.clone(), location))
                        }
                    }
                    _ => Err(syntax_error(
                        &location,
                        format!("Unexpected token: {:?}", token.kind),
                    )),
                }
            }
            None => Err(syntax_error(
                &Location {
                    file: "unknown".to_string(),
                    line: 0,
                    column: 0,
                },
                "Unexpected end of file",
            )),
        }
    }
}

