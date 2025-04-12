use std::env;
use std::fs::File;
use std::io::Write;
use tinyphp_rs::asmgen::AsmGenerator;
use tinyphp_rs::codegen::Instruction;

fn main() {
    // Create a simple set of instructions
    let instructions = vec![
        Instruction::PushInt(10),
        Instruction::StoreVar("a".to_string()),
        Instruction::PushInt(20),
        Instruction::StoreVar("b".to_string()),
        Instruction::LoadVar("a".to_string()),
        Instruction::LoadVar("b".to_string()),
        Instruction::Add,
        Instruction::StoreVar("c".to_string()),
        Instruction::PushString("The sum is: ".to_string()),
        Instruction::Echo,
        Instruction::LoadVar("c".to_string()),
        Instruction::Echo,
    ];

    // Generate assembly code
    let mut asmgen = AsmGenerator::new();
    let asm_code = asmgen.generate(&instructions);

    // Write to a file
    let output_file = "test_direct.s";
    let mut file = File::create(output_file).unwrap();
    file.write_all(asm_code.as_bytes()).unwrap();
    println!("Assembly code written to {}", output_file);

    // Compile with GCC
    let status = std::process::Command::new("gcc")
        .args(["-o", "test_direct.exe", output_file])
        .status()
        .unwrap();

    if status.success() {
        println!("Successfully compiled to test_direct.exe");
    } else {
        eprintln!("GCC compilation failed with exit code: {}", status);
    }
}
