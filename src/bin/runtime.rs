use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

/// Value type for the virtual machine
#[derive(Debug, Clone)]
enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "NULL"),
        }
    }
}

/// Simple virtual machine to execute PHP bytecode
struct VirtualMachine {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    functions: HashMap<String, Vec<String>>, // Function name -> instructions
}

impl VirtualMachine {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Execute a bytecode instruction
    fn execute_instruction(&mut self, instruction: &str) -> io::Result<()> {
        // Parse the instruction
        if instruction.starts_with("PushInt(") {
            let value = instruction
                .trim_start_matches("PushInt(")
                .trim_end_matches(")")
                .parse::<i64>()
                .unwrap();
            self.stack.push(Value::Integer(value));
        } else if instruction.starts_with("PushFloat(") {
            let value = instruction
                .trim_start_matches("PushFloat(")
                .trim_end_matches(")")
                .parse::<f64>()
                .unwrap();
            self.stack.push(Value::Float(value));
        } else if instruction.starts_with("PushString(") {
            let value = instruction
                .trim_start_matches("PushString(\"")
                .trim_end_matches("\")")
                .to_string();
            self.stack.push(Value::String(value));
        } else if instruction.starts_with("PushBool(") {
            let value = instruction
                .trim_start_matches("PushBool(")
                .trim_end_matches(")")
                .parse::<bool>()
                .unwrap();
            self.stack.push(Value::Boolean(value));
        } else if instruction == "PushNull" {
            self.stack.push(Value::Null);
        } else if instruction == "Pop" {
            self.stack.pop();
        } else if instruction.starts_with("LoadVar(") {
            let name = instruction
                .trim_start_matches("LoadVar(\"")
                .trim_end_matches("\")")
                .to_string();
            let value = self.variables.get(&name).cloned().unwrap_or(Value::Null);
            self.stack.push(value);
        } else if instruction.starts_with("StoreVar(") {
            let name = instruction
                .trim_start_matches("StoreVar(\"")
                .trim_end_matches("\")")
                .to_string();
            if let Some(value) = self.stack.last().cloned() {
                self.variables.insert(name, value);
            }
        } else if instruction == "Add" {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Integer(a + b));
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        self.stack.push(Value::Float(a + b));
                    }
                    (Value::Integer(a), Value::Float(b)) => {
                        self.stack.push(Value::Float(a as f64 + b));
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        self.stack.push(Value::Float(a + b as f64));
                    }
                    (Value::String(a), Value::String(b)) => {
                        self.stack.push(Value::String(a + &b));
                    }
                    _ => {
                        // PHP-like type coercion
                        self.stack.push(Value::Integer(0));
                    }
                }
            }
        } else if instruction == "Subtract" {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Integer(a - b));
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        self.stack.push(Value::Float(a - b));
                    }
                    (Value::Integer(a), Value::Float(b)) => {
                        self.stack.push(Value::Float(a as f64 - b));
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        self.stack.push(Value::Float(a - b as f64));
                    }
                    _ => {
                        // PHP-like type coercion
                        self.stack.push(Value::Integer(0));
                    }
                }
            }
        } else if instruction == "Multiply" {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Integer(a * b));
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        self.stack.push(Value::Float(a * b));
                    }
                    (Value::Integer(a), Value::Float(b)) => {
                        self.stack.push(Value::Float(a as f64 * b));
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        self.stack.push(Value::Float(a * b as f64));
                    }
                    _ => {
                        // PHP-like type coercion
                        self.stack.push(Value::Integer(0));
                    }
                }
            }
        } else if instruction == "Divide" {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b != 0 {
                            self.stack.push(Value::Integer(a / b));
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b != 0.0 {
                            self.stack.push(Value::Float(a / b));
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    (Value::Integer(a), Value::Float(b)) => {
                        if b != 0.0 {
                            self.stack.push(Value::Float(a as f64 / b));
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        if b != 0 {
                            self.stack.push(Value::Float(a / b as f64));
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    _ => {
                        // PHP-like type coercion
                        self.stack.push(Value::Integer(0));
                    }
                }
            }
        } else if instruction == "Greater" {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                match (a, b) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        self.stack.push(Value::Boolean(a > b));
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        self.stack.push(Value::Boolean(a > b));
                    }
                    (Value::Integer(a), Value::Float(b)) => {
                        self.stack.push(Value::Boolean((a as f64) > b));
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        self.stack.push(Value::Boolean(a > (b as f64)));
                    }
                    _ => {
                        // PHP-like type coercion
                        self.stack.push(Value::Boolean(false));
                    }
                }
            }
        } else if instruction == "Concat" {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                let a_str = match a {
                    Value::String(s) => s,
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Null => "NULL".to_string(),
                };

                let b_str = match b {
                    Value::String(s) => s,
                    Value::Integer(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Null => "NULL".to_string(),
                };

                self.stack.push(Value::String(a_str + &b_str));
            }
        } else if instruction == "Echo" {
            if let Some(value) = self.stack.pop() {
                println!("{}", value);
            }
        } else if instruction.starts_with("Call(") {
            // For simplicity, we'll just handle built-in functions
            let parts = instruction
                .trim_start_matches("Call(")
                .trim_end_matches(")")
                .split(", ")
                .collect::<Vec<_>>();

            if parts.len() == 2 {
                let function_name = parts[0].trim_matches('"');
                let arg_count = parts[1].parse::<usize>().unwrap_or(0);

                // Get arguments from the stack
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    if let Some(arg) = self.stack.pop() {
                        args.push(arg);
                    }
                }
                args.reverse(); // Arguments are popped in reverse order

                // Call the function
                match function_name {
                    "add" => {
                        if args.len() == 2 {
                            match (&args[0], &args[1]) {
                                (Value::Integer(a), Value::Integer(b)) => {
                                    self.stack.push(Value::Integer(a + b));
                                }
                                _ => {
                                    // PHP-like type coercion
                                    self.stack.push(Value::Integer(0));
                                }
                            }
                        }
                    }
                    _ => {
                        // Unknown function
                        self.stack.push(Value::Null);
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a bytecode file
    fn execute_file(&mut self, file_path: &str) -> io::Result<()> {
        // Open the file
        let mut file = File::open(file_path)?;

        // Read the header
        let mut header = [0; 8];
        file.read_exact(&mut header)?;

        // Check the magic number
        if &header[0..7] != b"TINYPHP" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid file format",
            ));
        }

        // Read the number of instructions
        let mut num_instructions_bytes = [0; 4];
        file.read_exact(&mut num_instructions_bytes)?;
        let num_instructions = u32::from_le_bytes(num_instructions_bytes);

        // Read and execute each instruction
        let mut instruction_buffer = Vec::new();
        for _ in 0..num_instructions {
            // Read until null terminator
            let mut instruction = Vec::new();
            let mut byte = [0; 1];

            loop {
                file.read_exact(&mut byte)?;
                if byte[0] == 0 {
                    break;
                }
                instruction.push(byte[0]);
            }

            // Convert to string and execute
            let instruction_str = String::from_utf8_lossy(&instruction).to_string();
            instruction_buffer.push(instruction_str);
        }

        // Execute all instructions
        let mut pc = 0; // Program counter
        while pc < instruction_buffer.len() {
            let instruction = &instruction_buffer[pc];

            // Handle jump instructions
            if instruction.starts_with("Jump(") {
                let target = instruction
                    .trim_start_matches("Jump(")
                    .trim_end_matches(")")
                    .parse::<usize>()
                    .unwrap_or(0);
                pc = target;
                continue;
            } else if instruction.starts_with("JumpIfFalse(") {
                let target = instruction
                    .trim_start_matches("JumpIfFalse(")
                    .trim_end_matches(")")
                    .parse::<usize>()
                    .unwrap_or(0);

                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Boolean(false) => {
                            pc = target;
                            continue;
                        },
                        Value::Integer(0) => {
                            pc = target;
                            continue;
                        },
                        Value::Null => {
                            pc = target;
                            continue;
                        },
                        _ => {}
                    }
                }
            } else if instruction.starts_with("JumpIfTrue(") {
                let target = instruction
                    .trim_start_matches("JumpIfTrue(")
                    .trim_end_matches(")")
                    .parse::<usize>()
                    .unwrap_or(0);

                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Boolean(true) => {
                            pc = target;
                            continue;
                        },
                        Value::Integer(i) if i != 0 => {
                            pc = target;
                            continue;
                        },
                        Value::Float(f) if f != 0.0 => {
                            pc = target;
                            continue;
                        },
                        Value::String(s) if !s.is_empty() => {
                            pc = target;
                            continue;
                        },
                        _ => {}
                    }
                }
            }

            // Print the instruction and program counter for debugging
            println!("[{}] Executing instruction: {}", pc, instruction);

            // Execute the instruction
            self.execute_instruction(instruction)?;

            // Move to the next instruction
            pc += 1;
        }

        Ok(())
    }
}

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <bytecode_file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    // Create a virtual machine
    let mut vm = VirtualMachine::new();

    // Execute the bytecode file
    match vm.execute_file(file_path) {
        Ok(_) => {
            println!("Program executed successfully");
        }
        Err(err) => {
            eprintln!("Error executing program: {}", err);
            process::exit(1);
        }
    }
}
