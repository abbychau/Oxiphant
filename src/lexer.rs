use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use lazy_static::lazy_static;

use crate::ast::Location;
use crate::error::{lexical_error, Result};

/// Represents a token in the PHP language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // PHP opening and closing tags
    PhpOpen,      // <?php
    PhpClose,     // ?>

    // Keywords
    Echo,
    If,
    Else,
    ElseIf,
    While,
    For,
    Foreach,
    As,
    Function,
    Return,
    True,
    False,
    Null,
    And,
    Or,
    Not,

    // Identifiers and literals
    Identifier(String),
    Variable(String),     // $name
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // Operators
    Plus,           // +
    Minus,          // -
    Asterisk,       // *
    Slash,          // /
    Percent,        // %
    Equal,          // ==
    NotEqual,       // !=
    Identical,      // ===
    NotIdentical,   // !==
    LessThan,       // <
    LessThanEqual,  // <=
    GreaterThan,    // >
    GreaterThanEqual, // >=
    LogicalAnd,     // &&
    LogicalOr,      // ||
    LogicalNot,     // !
    Concat,         // .
    Assign,         // =
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
    ModuloAssign,   // %=
    ConcatAssign,   // .=

    // Punctuation
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Semicolon,      // ;
    Comma,          // ,
    Colon,          // :
    DoubleColon,    // ::
    Arrow,          // ->
    DoubleArrow,    // =>
    QuestionMark,   // ?

    // End of file
    Eof,
}

/// Token with location information
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
}

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = {
        let mut m = HashMap::new();
        m.insert("echo", TokenKind::Echo);
        m.insert("if", TokenKind::If);
        m.insert("else", TokenKind::Else);
        m.insert("elseif", TokenKind::ElseIf);
        m.insert("while", TokenKind::While);
        m.insert("for", TokenKind::For);
        m.insert("foreach", TokenKind::Foreach);
        m.insert("as", TokenKind::As);
        m.insert("function", TokenKind::Function);
        m.insert("return", TokenKind::Return);
        m.insert("true", TokenKind::True);
        m.insert("false", TokenKind::False);
        m.insert("null", TokenKind::Null);
        m.insert("and", TokenKind::And);
        m.insert("or", TokenKind::Or);
        m.insert("not", TokenKind::Not);
        m
    };
}

/// Lexer for PHP source code
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    file: String,
    line: usize,
    column: usize,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, file: String) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            file,
            line: 1,
            column: 1,
            position: 0,
        }
    }

    /// Tokenize the source code
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        // Skip any HTML content before <?php
        self.skip_until_php_open_tag()?;

        // Add the PHP open tag
        tokens.push(Token {
            kind: TokenKind::PhpOpen,
            location: Location {
                file: self.file.clone(),
                line: self.line,
                column: if self.column >= 5 { self.column - 5 } else { 1 }, // "<?php" is 5 characters
            },
        });

        // Tokenize PHP code
        loop {
            // Skip whitespace
            self.skip_whitespace();

            // Check for end of file
            if self.chars.peek().is_none() {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    location: Location {
                        file: self.file.clone(),
                        line: self.line,
                        column: self.column,
                    },
                });
                break;
            }

            // Check for PHP closing tag
            if self.check_php_close_tag() {
                tokens.push(Token {
                    kind: TokenKind::PhpClose,
                    location: Location {
                        file: self.file.clone(),
                        line: self.line,
                        column: self.column,
                    },
                });

                // Skip any HTML content and look for the next PHP open tag
                if self.skip_until_php_open_tag()? {
                    tokens.push(Token {
                        kind: TokenKind::PhpOpen,
                        location: Location {
                            file: self.file.clone(),
                            line: self.line,
                            column: self.column - 5, // "<?php" is 5 characters
                        },
                    });
                } else {
                    // No more PHP code
                    tokens.push(Token {
                        kind: TokenKind::Eof,
                        location: Location {
                            file: self.file.clone(),
                            line: self.line,
                            column: self.column,
                        },
                    });
                    break;
                }
                continue;
            }

            // Get the current character
            let c = *self.chars.peek().unwrap();

            // Create a token based on the current character
            let token = match c {
                // Variable
                '$' => self.tokenize_variable()?,

                // Identifier
                'a'..='z' | 'A'..='Z' | '_' => self.tokenize_identifier()?,

                // Number
                '0'..='9' => self.tokenize_number()?,

                // String
                '"' | '\'' => self.tokenize_string()?,

                // Operators and punctuation
                '+' => self.tokenize_plus(),
                '-' => self.tokenize_minus(),
                '*' => self.tokenize_asterisk(),
                '/' => {
                    // Check for comments
                    if self.peek_next() == Some('/') {
                        self.skip_line_comment();
                        continue;
                    } else if self.peek_next() == Some('*') {
                        self.skip_block_comment()?;
                        continue;
                    } else {
                        self.tokenize_slash()
                    }
                },
                '%' => self.tokenize_percent(),
                '=' => self.tokenize_equals(),
                '!' => self.tokenize_exclamation(),
                '<' => self.tokenize_less_than(),
                '>' => self.tokenize_greater_than(),
                '&' => self.tokenize_ampersand(),
                '|' => self.tokenize_pipe(),
                '.' => self.tokenize_dot(),
                '(' => self.tokenize_single(TokenKind::LeftParen),
                ')' => self.tokenize_single(TokenKind::RightParen),
                '{' => self.tokenize_single(TokenKind::LeftBrace),
                '}' => self.tokenize_single(TokenKind::RightBrace),
                '[' => self.tokenize_single(TokenKind::LeftBracket),
                ']' => self.tokenize_single(TokenKind::RightBracket),
                ';' => self.tokenize_single(TokenKind::Semicolon),
                ',' => self.tokenize_single(TokenKind::Comma),
                ':' => self.tokenize_colon(),
                '?' => self.tokenize_single(TokenKind::QuestionMark),

                // Invalid character
                _ => {
                    return Err(lexical_error(
                        &Location {
                            file: self.file.clone(),
                            line: self.line,
                            column: self.column,
                        },
                        format!("Invalid character: '{}'", c),
                    ));
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Advance to the next character
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();

        if let Some(c) = c {
            self.position += 1;
            self.column += 1;

            if c == '\n' {
                self.line += 1;
                self.column = 1;
            }
        }

        c
    }

    /// Peek at the next character without advancing
    fn peek_next(&mut self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next(); // Skip current
        iter.next()  // Get next
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a line comment (// ...)
    fn skip_line_comment(&mut self) {
        // Skip the //
        self.advance();
        self.advance();

        // Skip until end of line or end of file
        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Skip a block comment (/* ... */)
    fn skip_block_comment(&mut self) -> Result<()> {
        // Skip the /*
        self.advance();
        self.advance();

        // Skip until */ or end of file
        while let Some(c) = self.advance() {
            if c == '*' && self.chars.peek() == Some(&'/') {
                self.advance(); // Skip the /
                return Ok(());
            }
        }

        Err(lexical_error(
            &Location {
                file: self.file.clone(),
                line: self.line,
                column: self.column,
            },
            "Unterminated block comment",
        ))
    }

    /// Check for PHP closing tag
    fn check_php_close_tag(&mut self) -> bool {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;

        // Check for ?>
        if self.chars.peek() == Some(&'?') && self.peek_next() == Some('>') {
            self.advance(); // Skip ?
            self.advance(); // Skip >
            return true;
        }

        // Reset position if not a closing tag
        self.position = start_pos;
        self.line = start_line;
        self.column = start_column;
        self.chars = self.source[start_pos..].chars().peekable();

        false
    }

    /// Skip until PHP open tag (<?php)
    /// Returns true if an open tag was found, false if end of file
    fn skip_until_php_open_tag(&mut self) -> Result<bool> {
        while let Some(&c) = self.chars.peek() {
            if c == '<' {
                let start_pos = self.position;
                let start_line = self.line;
                let start_column = self.column;

                self.advance(); // Skip <

                if self.chars.peek() == Some(&'?') {
                    self.advance(); // Skip ?

                    // Check for "php"
                    if self.chars.peek() == Some(&'p') {
                        self.advance(); // Skip p

                        if self.chars.peek() == Some(&'h') {
                            self.advance(); // Skip h

                            if self.chars.peek() == Some(&'p') {
                                self.advance(); // Skip p

                                // Skip whitespace after <?php
                                self.skip_whitespace();

                                return Ok(true);
                            }
                        }
                    }
                }

                // Reset position if not an open tag
                self.position = start_pos;
                self.line = start_line;
                self.column = start_column;
                self.chars = self.source[start_pos..].chars().peekable();
            }

            self.advance();
        }

        // End of file
        Ok(false)
    }

    /// Tokenize a variable ($name)
    fn tokenize_variable(&mut self) -> Result<Token> {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip $

        let mut name = String::new();

        // First character must be a letter or underscore
        if let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() || c == '_' {
                name.push(c);
                self.advance();
            } else {
                return Err(lexical_error(
                    &location,
                    "Variable name must start with a letter or underscore",
                ));
            }
        } else {
            return Err(lexical_error(
                &location,
                "Unexpected end of file after $",
            ));
        }

        // Rest of the characters can be alphanumeric or underscore
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token {
            kind: TokenKind::Variable(name),
            location,
        })
    }

    /// Tokenize an identifier
    fn tokenize_identifier(&mut self) -> Result<Token> {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        let mut name = String::new();

        // First character is already checked to be a letter or underscore
        name.push(*self.chars.peek().unwrap());
        self.advance();

        // Rest of the characters can be alphanumeric or underscore
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        let kind = if let Some(keyword) = KEYWORDS.get(name.as_str()) {
            keyword.clone()
        } else {
            TokenKind::Identifier(name)
        };

        Ok(Token {
            kind,
            location,
        })
    }

    /// Tokenize a number (integer or float)
    fn tokenize_number(&mut self) -> Result<Token> {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        let mut number = String::new();
        let mut is_float = false;

        // Parse digits before decimal point
        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) {
                number.push(c);
                self.advance();
            } else if c == '.' {
                is_float = true;
                number.push(c);
                self.advance();
                break;
            } else {
                break;
            }
        }

        // Parse digits after decimal point if it's a float
        if is_float {
            while let Some(&c) = self.chars.peek() {
                if c.is_digit(10) {
                    number.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Parse the number
        if is_float {
            match number.parse::<f64>() {
                Ok(value) => Ok(Token {
                    kind: TokenKind::FloatLiteral(value),
                    location,
                }),
                Err(_) => Err(lexical_error(
                    &location,
                    format!("Invalid float literal: {}", number),
                )),
            }
        } else {
            match number.parse::<i64>() {
                Ok(value) => Ok(Token {
                    kind: TokenKind::IntLiteral(value),
                    location,
                }),
                Err(_) => Err(lexical_error(
                    &location,
                    format!("Invalid integer literal: {}", number),
                )),
            }
        }
    }

    /// Tokenize a string literal
    fn tokenize_string(&mut self) -> Result<Token> {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        let quote = self.advance().unwrap(); // Get the quote character (' or ")
        let mut value = String::new();

        // Parse until closing quote
        while let Some(&c) = self.chars.peek() {
            if c == quote {
                self.advance(); // Skip closing quote
                break;
            } else if c == '\\' {
                // Handle escape sequences
                self.advance(); // Skip backslash

                if let Some(&next) = self.chars.peek() {
                    match next {
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        '\\' => value.push('\\'),
                        '\'' => value.push('\''),
                        '"' => value.push('"'),
                        '$' => value.push('$'), // PHP escapes $ in strings
                        _ => value.push(next),
                    }
                    self.advance();
                } else {
                    return Err(lexical_error(
                        &location,
                        "Unexpected end of file in string literal",
                    ));
                }
            } else {
                value.push(c);
                self.advance();
            }
        }

        Ok(Token {
            kind: TokenKind::StringLiteral(value),
            location,
        })
    }

    /// Tokenize a single character token
    fn tokenize_single(&mut self, kind: TokenKind) -> Token {
        let token = Token {
            kind,
            location: Location {
                file: self.file.clone(),
                line: self.line,
                column: self.column,
            },
        };

        self.advance();
        token
    }

    /// Tokenize plus (+) or plus equals (+=)
    fn tokenize_plus(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip +

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::PlusAssign,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Plus,
                location,
            }
        }
    }

    /// Tokenize minus (-) or minus equals (-=) or arrow (->)
    fn tokenize_minus(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip -

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::MinusAssign,
                location,
            }
        } else if self.chars.peek() == Some(&'>') {
            self.advance(); // Skip >
            Token {
                kind: TokenKind::Arrow,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Minus,
                location,
            }
        }
    }

    /// Tokenize asterisk (*) or multiply equals (*=)
    fn tokenize_asterisk(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip *

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::MultiplyAssign,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Asterisk,
                location,
            }
        }
    }

    /// Tokenize slash (/) or divide equals (/=)
    fn tokenize_slash(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip /

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::DivideAssign,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Slash,
                location,
            }
        }
    }

    /// Tokenize percent (%) or modulo equals (%=)
    fn tokenize_percent(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip %

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::ModuloAssign,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Percent,
                location,
            }
        }
    }

    /// Tokenize equals (=) or equal (==) or identical (===)
    fn tokenize_equals(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip =

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =

            if self.chars.peek() == Some(&'=') {
                self.advance(); // Skip =
                Token {
                    kind: TokenKind::Identical,
                    location,
                }
            } else {
                Token {
                    kind: TokenKind::Equal,
                    location,
                }
            }
        } else if self.chars.peek() == Some(&'>') {
            self.advance(); // Skip >
            Token {
                kind: TokenKind::DoubleArrow,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Assign,
                location,
            }
        }
    }

    /// Tokenize exclamation (!) or not equal (!=) or not identical (!==)
    fn tokenize_exclamation(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip !

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =

            if self.chars.peek() == Some(&'=') {
                self.advance(); // Skip =
                Token {
                    kind: TokenKind::NotIdentical,
                    location,
                }
            } else {
                Token {
                    kind: TokenKind::NotEqual,
                    location,
                }
            }
        } else {
            Token {
                kind: TokenKind::LogicalNot,
                location,
            }
        }
    }

    /// Tokenize less than (<) or less than or equal (<=)
    fn tokenize_less_than(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip <

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::LessThanEqual,
                location,
            }
        } else {
            Token {
                kind: TokenKind::LessThan,
                location,
            }
        }
    }

    /// Tokenize greater than (>) or greater than or equal (>=)
    fn tokenize_greater_than(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip >

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::GreaterThanEqual,
                location,
            }
        } else {
            Token {
                kind: TokenKind::GreaterThan,
                location,
            }
        }
    }

    /// Tokenize ampersand (&) or logical and (&&)
    fn tokenize_ampersand(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip &

        if self.chars.peek() == Some(&'&') {
            self.advance(); // Skip &
            Token {
                kind: TokenKind::LogicalAnd,
                location,
            }
        } else {
            // PHP doesn't use single & much, but we'll tokenize it anyway
            Token {
                kind: TokenKind::Asterisk, // Using Asterisk as a placeholder for BitwiseAnd
                location,
            }
        }
    }

    /// Tokenize pipe (|) or logical or (||)
    fn tokenize_pipe(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip |

        if self.chars.peek() == Some(&'|') {
            self.advance(); // Skip |
            Token {
                kind: TokenKind::LogicalOr,
                location,
            }
        } else {
            // PHP doesn't use single | much, but we'll tokenize it anyway
            Token {
                kind: TokenKind::Plus, // Using Plus as a placeholder for BitwiseOr
                location,
            }
        }
    }

    /// Tokenize dot (.) or concat equals (.=)
    fn tokenize_dot(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip .

        if self.chars.peek() == Some(&'=') {
            self.advance(); // Skip =
            Token {
                kind: TokenKind::ConcatAssign,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Concat,
                location,
            }
        }
    }

    /// Tokenize colon (:) or double colon (::)
    fn tokenize_colon(&mut self) -> Token {
        let location = Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
        };

        self.advance(); // Skip :

        if self.chars.peek() == Some(&':') {
            self.advance(); // Skip :
            Token {
                kind: TokenKind::DoubleColon,
                location,
            }
        } else {
            Token {
                kind: TokenKind::Colon,
                location,
            }
        }
    }
}
