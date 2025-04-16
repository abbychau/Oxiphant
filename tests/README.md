# Oxiphant Testing

This directory contains test scripts and utilities for testing the Oxiphant compiler.

## Directory Structure

- `scripts/`: Contains PHP test scripts
- `output/`: Contains compiled executables and assembly files

## Test Scripts

### compile_test.ps1

Compiles a PHP script to an executable using GCC.

```powershell
.\compile_test.ps1 <script_name>
```

Example:
```powershell
.\compile_test.ps1 simple_print
```

This will:
1. Compile `tests\scripts\simple_print.php` to `tests\output\simple_print.exe`
2. Save the generated assembly code to `tests\output\simple_print.s`

### run_test.ps1

Runs a compiled PHP script.

```powershell
.\run_test.ps1 <script_name>
```

Example:
```powershell
.\run_test.ps1 simple_print
```

This will run `tests\output\simple_print.exe`.

### test_all.ps1

Compiles and runs all PHP scripts in the `scripts/` directory.

```powershell
.\test_all.ps1
```

## Available Test Scripts

### Basic Tests
1. `simple_print.php`: Prints a string and a number
2. `test_add.php`: Tests addition operation
3. `test_subtract.php`: Tests subtraction operation
4. `test_multiply.php`: Tests multiplication operation

### Simple Tests
5. `test_simple_math.php`: Tests basic arithmetic operations (addition, subtraction, multiplication, division)
6. `test_simple_string.php`: Tests basic string operations and concatenation
7. `test_simple_if.php`: Tests simple if/else statements
8. `test_simple_while.php`: Tests simple while loops

### Advanced Tests
9. `test_if_else.php`: Tests more complex conditional statements
10. `test_while_loop.php`: Tests more complex loop structures
11. `test_expressions.php`: Tests nested expressions and operator precedence
12. `test_calculations.php`: Tests complex calculations with multiple variables
13. `test_strings.php`: Tests string operations and concatenation

## Adding New Tests

To add a new test:

1. Create a new PHP file in the `scripts/` directory
2. Run `.\compile_test.ps1 <your_script_name>` to compile it
3. Run `.\run_test.ps1 <your_script_name>` to run it

Or simply run `.\test_all.ps1` to compile and run all tests.
