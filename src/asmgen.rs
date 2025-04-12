// asmgen is responsible for generating assembly code from bytecode instructions

use crate::codegen::Instruction;
use std::fmt::Write;

/// Assembly code generator
pub struct AsmGenerator {
    asm_code: String,
    string_literals: Vec<String>,
    label_counter: usize,
    variables: std::collections::HashMap<String, usize>,
    var_counter: usize,
}

impl AsmGenerator {
    /// Create a new assembly code generator
    pub fn new() -> Self {
        Self {
            asm_code: String::new(),
            string_literals: Vec::new(),
            label_counter: 0,
            variables: std::collections::HashMap::new(),
            var_counter: 0,
        }
    }

    /// Generate assembly code from bytecode instructions
    pub fn generate(&mut self, instructions: &[Instruction]) -> String {
        // Clear previous state
        self.asm_code.clear();
        self.string_literals.clear();
        self.label_counter = 0;
        self.variables.clear();
        self.var_counter = 0;

        // Add assembly header
        self.add_header();

        // Process instructions
        for instruction in instructions {
            self.process_instruction(instruction);
        }

        // Add string literals section
        self.add_string_literals();

        // Add assembly footer
        self.add_footer();

        self.asm_code.clone()
    }

    /// Add assembly header
    fn add_header(&mut self) {
        // Windows x64 calling convention
        writeln!(self.asm_code, ".intel_syntax noprefix").unwrap();
        writeln!(self.asm_code, ".text").unwrap();

        // External functions
        writeln!(self.asm_code, ".extern printf").unwrap();
        writeln!(self.asm_code, ".extern putchar").unwrap();
        writeln!(self.asm_code, ".extern sprintf").unwrap();

        // Main function
        writeln!(self.asm_code, ".global main").unwrap();
        writeln!(self.asm_code, "main:").unwrap();
        writeln!(self.asm_code, "    push rbp").unwrap();
        writeln!(self.asm_code, "    mov rbp, rsp").unwrap();
        writeln!(self.asm_code, "    sub rsp, 256  # Reserve stack space for variables").unwrap();
        // Windows x64 requires 32 bytes of shadow space
        writeln!(self.asm_code, "    sub rsp, 32   # Shadow space for Windows x64").unwrap();
        writeln!(self.asm_code).unwrap();
    }

    /// Add assembly footer
    fn add_footer(&mut self) {
        writeln!(self.asm_code, "    # Program exit").unwrap();
        writeln!(self.asm_code, "    add rsp, 32   # Restore shadow space").unwrap();
        writeln!(self.asm_code, "    mov rax, 0  # Return 0").unwrap();
        writeln!(self.asm_code, "    leave").unwrap();
        writeln!(self.asm_code, "    ret").unwrap();
    }

    /// Add string literals section
    fn add_string_literals(&mut self) {
        writeln!(self.asm_code, ".data").unwrap();

        // Format string for printf
        writeln!(self.asm_code, "fmt_str:").unwrap();
        writeln!(self.asm_code, "    .string \"%s\"").unwrap();
        writeln!(self.asm_code, "fmt_int:").unwrap();
        writeln!(self.asm_code, "    .string \"%d\"").unwrap();
        writeln!(self.asm_code, "fmt_float:").unwrap();
        writeln!(self.asm_code, "    .string \"%f\"").unwrap();

        // Add string literals
        for (i, s) in self.string_literals.iter().enumerate() {
            let escaped = s.replace("\"", "\\\"");
            writeln!(self.asm_code, "str_{}:", i).unwrap();
            writeln!(self.asm_code, "    .string \"{}\"", escaped).unwrap();
        }
    }

    /// Generate a new label
    fn new_label(&mut self) -> String {
        let label = format!("label_{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Get the stack offset for a variable
    fn get_var_offset(&mut self, name: &str) -> usize {
        // If the variable doesn't exist, allocate a new offset
        if !self.variables.contains_key(name) {
            // Each variable takes 8 bytes (64-bit)
            // Start at offset 8 to account for saved rbp
            let offset = 8 + (self.var_counter * 8);
            self.variables.insert(name.to_string(), offset);
            self.var_counter += 1;
        }

        // Return the offset
        *self.variables.get(name).unwrap()
    }

    /// Process a single instruction
    fn process_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::PushInt(value) => {
                writeln!(self.asm_code, "    # PushInt({})", value).unwrap();
                writeln!(self.asm_code, "    mov rax, {}", value).unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::PushFloat(value) => {
                writeln!(self.asm_code, "    # PushFloat({})", value).unwrap();
                // For simplicity, we'll just convert to int
                writeln!(self.asm_code, "    mov rax, {}", *value as i64).unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::PushString(value) => {
                writeln!(self.asm_code, "    # PushString(\"{}\")", value).unwrap();
                let str_index = self.string_literals.len();
                self.string_literals.push(value.clone());
                writeln!(self.asm_code, "    lea rax, [rip + str_{}]", str_index).unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::PushBool(value) => {
                writeln!(self.asm_code, "    # PushBool({})", value).unwrap();
                writeln!(self.asm_code, "    mov rax, {}", if *value { 1 } else { 0 }).unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::PushNull => {
                writeln!(self.asm_code, "    # PushNull").unwrap();
                writeln!(self.asm_code, "    mov rax, 0").unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::Pop => {
                writeln!(self.asm_code, "    # Pop").unwrap();
                writeln!(self.asm_code, "    add rsp, 8").unwrap();
            }
            Instruction::CreateArray => {
                writeln!(self.asm_code, "    # CreateArray").unwrap();
                // For simplicity, we'll just allocate a fixed-size array on the stack
                // In a real implementation, we would need to allocate memory on the heap
                writeln!(self.asm_code, "    sub rsp, 64  # Allocate space for array").unwrap();
                writeln!(self.asm_code, "    mov rax, rsp  # Store array pointer").unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::ArrayPush => {
                writeln!(self.asm_code, "    # ArrayPush").unwrap();
                // For simplicity, we'll just store the value at a fixed offset
                // In a real implementation, we would need to track the array size
                writeln!(self.asm_code, "    pop rax  # Value to push").unwrap();
                writeln!(self.asm_code, "    pop rdx  # Array pointer").unwrap();
                writeln!(self.asm_code, "    mov [rdx], rax  # Store value in array").unwrap();
                writeln!(self.asm_code, "    push rdx  # Push array pointer back").unwrap();
            }
            Instruction::ArraySet => {
                writeln!(self.asm_code, "    # ArraySet").unwrap();
                // For simplicity, we'll just store the value at the key offset
                // In a real implementation, we would need to handle string keys
                writeln!(self.asm_code, "    pop rdx  # Key").unwrap();
                writeln!(self.asm_code, "    pop rax  # Value").unwrap();
                writeln!(self.asm_code, "    pop rcx  # Array pointer").unwrap();
                writeln!(self.asm_code, "    mov [rcx + rdx * 8], rax  # Store value at key offset").unwrap();
                writeln!(self.asm_code, "    push rcx  # Push array pointer back").unwrap();
            }
            Instruction::ArrayGet => {
                writeln!(self.asm_code, "    # ArrayGet").unwrap();
                // For simplicity, we'll just load the value at the key offset
                // In a real implementation, we would need to handle string keys
                writeln!(self.asm_code, "    pop rdx  # Key").unwrap();
                writeln!(self.asm_code, "    pop rcx  # Array pointer").unwrap();
                writeln!(self.asm_code, "    mov rax, [rcx + rdx * 8]  # Load value at key offset").unwrap();
                writeln!(self.asm_code, "    push rax  # Push value").unwrap();
            }
            Instruction::Add => {
                writeln!(self.asm_code, "    # Add").unwrap();
                writeln!(self.asm_code, "    pop rax  # Second operand").unwrap();
                writeln!(self.asm_code, "    pop rbx  # First operand").unwrap();
                writeln!(self.asm_code, "    add rax, rbx").unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::Subtract => {
                writeln!(self.asm_code, "    # Subtract").unwrap();
                writeln!(self.asm_code, "    pop rax  # Second operand").unwrap();
                writeln!(self.asm_code, "    pop rbx  # First operand").unwrap();
                writeln!(self.asm_code, "    sub rbx, rax").unwrap();
                writeln!(self.asm_code, "    push rbx").unwrap();
            }
            Instruction::Multiply => {
                writeln!(self.asm_code, "    # Multiply").unwrap();
                writeln!(self.asm_code, "    pop rax  # Second operand").unwrap();
                writeln!(self.asm_code, "    pop rbx  # First operand").unwrap();
                writeln!(self.asm_code, "    imul rbx").unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::Echo => {
                writeln!(self.asm_code, "    # Echo").unwrap();
                // Windows x64 calling convention: rcx, rdx, r8, r9
                writeln!(self.asm_code, "    pop rdx  # Value to print (second arg)").unwrap();

                // Check if it's a string or an integer
                writeln!(self.asm_code, "    # Check if it's a string or an integer").unwrap();
                writeln!(self.asm_code, "    cmp rdx, 100000  # Assume values < 100000 are integers").unwrap();
                writeln!(self.asm_code, "    jge .print_string_{}", self.label_counter).unwrap();

                // Print as integer
                writeln!(self.asm_code, "    # Print as integer").unwrap();
                writeln!(self.asm_code, "    lea rcx, [rip + fmt_int]  # Format string (first arg)").unwrap();
                writeln!(self.asm_code, "    mov rax, 0").unwrap();
                writeln!(self.asm_code, "    call printf").unwrap();
                writeln!(self.asm_code, "    jmp .echo_done_{}", self.label_counter).unwrap();

                // Print as string
                writeln!(self.asm_code, ".print_string_{}:", self.label_counter).unwrap();
                writeln!(self.asm_code, "    lea rcx, [rip + fmt_str]  # Format string (first arg)").unwrap();
                writeln!(self.asm_code, "    mov rax, 0").unwrap();
                writeln!(self.asm_code, "    call printf").unwrap();

                writeln!(self.asm_code, ".echo_done_{}:", self.label_counter).unwrap();
                self.label_counter += 1;

                // Add newline
                writeln!(self.asm_code, "    mov rcx, 10  # '\\n' (first arg)").unwrap();
                writeln!(self.asm_code, "    call putchar").unwrap();
            }
            Instruction::Concat => {
                writeln!(self.asm_code, "    # Concat").unwrap();
                // For simplicity, we'll just convert the second operand to a string and use it
                // In a real implementation, we would need to allocate memory and copy both strings
                writeln!(self.asm_code, "    pop rdx  # Second operand").unwrap();
                writeln!(self.asm_code, "    pop rcx  # First operand (string)").unwrap();

                // Check if second operand is an integer
                writeln!(self.asm_code, "    cmp rdx, 100000  # Assume values < 100000 are integers").unwrap();
                writeln!(self.asm_code, "    jge .concat_string_{}", self.label_counter).unwrap();

                // Convert integer to string and use it
                writeln!(self.asm_code, "    # Convert integer to string").unwrap();
                writeln!(self.asm_code, "    push rcx  # Save first operand").unwrap();
                writeln!(self.asm_code, "    mov rcx, [rip + fmt_int]  # Format string").unwrap();
                writeln!(self.asm_code, "    sub rsp, 64  # Reserve space for string buffer").unwrap();
                writeln!(self.asm_code, "    lea r8, [rsp]  # Buffer address").unwrap();
                writeln!(self.asm_code, "    mov r9, 64  # Buffer size").unwrap();
                writeln!(self.asm_code, "    mov rax, 0").unwrap();
                writeln!(self.asm_code, "    call sprintf").unwrap();
                writeln!(self.asm_code, "    lea rdx, [rsp]  # Get buffer address").unwrap();
                writeln!(self.asm_code, "    add rsp, 64  # Restore stack").unwrap();
                writeln!(self.asm_code, "    pop rcx  # Restore first operand").unwrap();

                writeln!(self.asm_code, "    jmp .concat_done_{}", self.label_counter).unwrap();

                // Use second operand as string
                writeln!(self.asm_code, ".concat_string_{}:", self.label_counter).unwrap();

                writeln!(self.asm_code, ".concat_done_{}:", self.label_counter).unwrap();
                self.label_counter += 1;

                // For now, just use the first string
                writeln!(self.asm_code, "    push rcx  # Push first string").unwrap();
            }
            Instruction::LoadVar(name) => {
                writeln!(self.asm_code, "    # LoadVar(\"{}\")", name).unwrap();
                // Get the variable offset
                let offset = self.get_var_offset(name);
                writeln!(self.asm_code, "    mov rax, [rbp - {}]  # Load variable", offset).unwrap();
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            Instruction::StoreVar(name) => {
                writeln!(self.asm_code, "    # StoreVar(\"{}\")", name).unwrap();
                // Get the variable offset
                let offset = self.get_var_offset(name);
                writeln!(self.asm_code, "    pop rax  # Value to store").unwrap();
                writeln!(self.asm_code, "    mov [rbp - {}], rax  # Store variable", offset).unwrap();
                // Push the value back on the stack (PHP assignment is an expression)
                writeln!(self.asm_code, "    push rax").unwrap();
            }
            // Simplified implementation for other instructions
            _ => {
                writeln!(self.asm_code, "    # Unimplemented: {:?}", instruction).unwrap();
            }
        }
    }
}
