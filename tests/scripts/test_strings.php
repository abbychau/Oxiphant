<?php
// Test string operations
echo "Testing string operations:";

// String concatenation
$firstName = "John";
$lastName = "Doe";
$fullName = $firstName . " " . $lastName;
echo "Full name: $fullName";

// String with numbers
$age = 30;
$message = "My name is " . $fullName . " and I am " . $age . " years old.";
echo $message;

// Multiple concatenations
$a = "Hello";
$b = "World";
$c = "!";
$greeting = $a . ", " . $b . $c;
echo $greeting;

// Concatenation with expressions
$num1 = 10;
$num2 = 20;
$result = $num1 + $num2;
$equation = $num1 . " + " . $num2 . " = " . $result;
echo $equation;

// Nested string expressions
$prefix = "Value: ";
$value = 42;
$suffix = " (The answer)";
$output = $prefix . $value . $suffix;
echo $output;
?>
