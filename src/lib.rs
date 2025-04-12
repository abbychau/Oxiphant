pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod typechecker;
pub mod codegen;
pub mod asmgen;

use std::fs;
use std::path::Path;


use crate::error::Result;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::typechecker::TypeChecker;
use crate::codegen::CodeGenerator;

/// Compile a PHP file to bytecode
pub fn compile_file<P: AsRef<Path>>(path: P) -> Result<Vec<codegen::Instruction>> {
    // Read the file
    let source = fs::read_to_string(path.as_ref())?;
    let file_name = path.as_ref().to_string_lossy().to_string();

    // Tokenize
    let mut lexer = Lexer::new(&source, file_name);
    let tokens = lexer.tokenize()?;

    // Parse
    let mut parser = Parser::new(&tokens);
    let ast = parser.parse_program()?;

    // Type check
    let mut typechecker = TypeChecker::new();
    typechecker.check_program(&ast)?;

    // Generate code
    let mut codegen = CodeGenerator::new();
    let instructions = codegen.generate(&ast)?;

    Ok(instructions)
}
