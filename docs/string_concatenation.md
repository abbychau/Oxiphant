# String Concatenation in TinyPHP

## Current Implementation

The current implementation of string concatenation in TinyPHP has some limitations:

1. **Dot Operator**: The dot (`.`) operator for string concatenation is not fully implemented. When using the dot operator, the output will be a placeholder string (`<concatenated string>`) instead of the actual concatenated string.

2. **Multiple Expressions**: Using multiple expressions in a single echo statement is the recommended approach for string concatenation. This approach works correctly and produces the expected output.

3. **Pre-concatenated Strings**: Using pre-concatenated strings is another approach that works correctly.

## Examples

### Using Dot Operator (Not Recommended)

```php
$a = "Hello, ";
$b = "World!";
echo $a.$b; // Outputs: <concatenated string>
```

### Using Multiple Expressions (Recommended)

```php
$a = "Hello, ";
$b = "World!";
echo $a, $b; // Outputs: Hello, World!
```

### Using Pre-concatenated Strings (Alternative)

```php
$c = "Hello, World!";
echo $c; // Outputs: Hello, World!
```

## Future Improvements

In future versions of TinyPHP, we plan to fully implement the dot operator for string concatenation. This will involve:

1. Properly allocating memory for the concatenated string
2. Copying the concatenated string from the stack buffer to the string literals section
3. Handling edge cases like empty strings, null values, etc.

Until then, please use the recommended approaches described above.
