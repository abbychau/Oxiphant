<?php
// Test while loops
echo "Testing while loops:";

// Simple while loop
$i = 1;
echo "Counting from 1 to 5:";
while ($i <= 5) {
    echo $i;
    $i = $i + 1;
}

// Nested while loops
echo "Multiplication table (1-3):";
$i = 1;
while ($i <= 3) {
    $j = 1;
    while ($j <= 3) {
        $result = $i * $j;
        echo "$i x $j = $result";
        $j = $j + 1;
    }
    $i = $i + 1;
}

// While loop with conditional break
echo "Loop with conditional break:";
$i = 1;
while ($i <= 10) {
    if ($i == 6) {
        echo "Breaking at i=6";
        $i = 11; // Simulate break
    } else {
        echo $i;
        $i = $i + 1;
    }
}
?>
