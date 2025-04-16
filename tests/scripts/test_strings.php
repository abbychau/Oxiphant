<?php
// Test string operations
echo "Testing string operations:";

// String concatenation
$firstName = "John";
$lastName = "Doe";
// Using multiple expressions in a single echo statement
echo "Full name: ", $firstName, " ", $lastName;

// String with numbers
$age = 30;
echo "My name is ", $firstName, " ", $lastName, " and I am ", $age, " years old.";

// Multiple concatenations
$a = "Hello";
$b = "World";
$c = "!";
echo $a, ", ", $b, $c;

// Concatenation with expressions
$num1 = 10;
$num2 = 20;
$result = $num1 + $num2;
echo $num1, " + ", $num2, " = ", $result;

// Nested string expressions
$prefix = "Value: ";
$value = 42;
$suffix = " (The answer)";
echo $prefix, $value, $suffix;
?>
