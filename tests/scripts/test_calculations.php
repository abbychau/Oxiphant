<?php
// Test complex calculations with multiple variables
echo "Testing complex calculations:";

// Calculate the area and perimeter of a rectangle
$length = 7;
$width = 5;
$area = $length * $width;
$perimeter = 2 * ($length + $width);

echo "Rectangle dimensions: $length x $width";
echo "Area: $area";
echo "Perimeter: $perimeter";

// Calculate the volume and surface area of a box
$height = 3;
$volume = $length * $width * $height;
$surfaceArea = 2 * ($length * $width + $length * $height + $width * $height);

echo "Box dimensions: $length x $width x $height";
echo "Volume: $volume";
echo "Surface Area: $surfaceArea";

// Calculate the average of 5 numbers
$num1 = 10;
$num2 = 20;
$num3 = 30;
$num4 = 40;
$num5 = 50;
$sum = $num1 + $num2 + $num3 + $num4 + $num5;
$average = $sum / 5;

echo "Numbers: $num1, $num2, $num3, $num4, $num5";
echo "Sum: $sum";
echo "Average: $average";

// Calculate the discriminant of a quadratic equation
$a = 1;
$b = -3;
$c = 2;
$discriminant = $b * $b - 4 * $a * $c;

echo "Quadratic equation: ${a}x^2 + ${b}x + $c";
echo "Discriminant: $discriminant";
?>
