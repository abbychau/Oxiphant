<?php
// Test simple while loop
echo "Testing simple while loop:";

// Count from 1 to 5
$i = 1;
echo "Counting from 1 to 5:";
while ($i <= 5) {
    echo $i;
    $i = $i + 1;
}

// Count down from 5 to 1
$i = 5;
echo "Counting down from 5 to 1:";
while ($i >= 1) {
    echo $i;
    $i = $i - 1;
}
?>
