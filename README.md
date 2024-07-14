# Unlox
An interpreter for the Lox programming language written in Rust.

## Example
```
var a = 0;
{
    var a = 5;
    a = a + 1;
    print a;
}
a = a + 1;
print a;
```
Output:
```
6
1
```
