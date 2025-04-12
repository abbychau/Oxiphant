<?php
// Test array operations
echo "Testing array operations:";

// Create an array
$arr = array(10, 20, 30);

// Access array elements
echo "First element:";
echo $arr[0];

echo "Second element:";
echo $arr[1];

echo "Third element:";
echo $arr[2];

// Modify array elements
$arr[1] = 25;
echo "Modified second element:";
echo $arr[1];

// Add a new element
$arr[3] = 40;
echo "New fourth element:";
echo $arr[3];

// Create an associative array
$person = array(
    "name" => "John",
    "age" => 30,
    "city" => "New York"
);

// Access associative array elements
echo "Person's name:";
echo $person["name"];

echo "Person's age:";
echo $person["age"];

echo "Person's city:";
echo $person["city"];
?>
