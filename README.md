# Unlox
An interpreter for the Lox programming language written in Rust.

## Example
Print 50th element in the Fibonacci sequence:
```
var start = clock();
var n = 50;

var a = 0;
var b = 1;

for (var i = 0; a < n; i = i + 1) {
    var temp = a;
    a = b;
    b = temp + b;
}

var end = clock();
print a;
print end - start;
```
Output:
```
12586269025
0.00004029273986816406;
```
