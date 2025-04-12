<?php
// Test conditional statements (if/else)
$a = 15;
$b = 10;

echo "Testing if/else statements:";

// Simple if condition
if ($a > $b) {
    echo "a is greater than b";
}

// If-else condition
$c = 5;
if ($c > $a) {
    echo "c is greater than a";
} else {
    echo "c is not greater than a";
}

// Multiple conditions
$d = 20;
if ($d < $a) {
    echo "d is less than a";
} else if ($d < $b) {
    echo "d is less than b";
} else {
    echo "d is greater than or equal to both a and b";
}

// Nested if statements
if ($a > $b) {
    if ($c < $b) {
        echo "a > b and c < b";
    } else {
        echo "a > b and c >= b";
    }
}
?>
