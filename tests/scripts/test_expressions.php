<?php
// Test complex expressions and operator precedence
echo "Testing complex expressions:";

// Arithmetic operator precedence
$a = 5;
$b = 3;
$c = 2;

$result1 = $a + $b * $c;
echo "5 + 3 * 2 = $result1";  // Should be 11 (multiplication before addition)

$result2 = ($a + $b) * $c;
echo "(5 + 3) * 2 = $result2";  // Should be 16 (parentheses first)

// Complex nested expressions
$d = 10;
$result3 = $a + $b * $c - $d / $c;
echo "5 + 3 * 2 - 10 / 2 = $result3";  // Should be 6 (5 + 6 - 5 = 6)

// Multiple operations
$result4 = $a * $b + $c * $d - $a;
echo "5 * 3 + 2 * 10 - 5 = $result4";  // Should be 30 (15 + 20 - 5 = 30)

// Nested parentheses
$result5 = (($a + $b) * ($c + $d)) / $c;
echo "((5 + 3) * (2 + 10)) / 2 = $result5";  // Should be 48 (8 * 12 / 2 = 48)
?>
