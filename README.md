# Unlox
An interpreter for the Lox programming language written in Rust.

## Example
Calculate 28th element in the Fibonacci sequence:
```
fun fib_iterative(n) {
    var a = 0;
    var b = 1;

    for (var i = 0; i < n; i = i + 1) {
        var temp = a;
        a = b;
        b = temp + b;
    }
    return a;
}

fun fib_recursive(n) {
    if (n <= 1) return n;
    return fib_recursive(n - 2) + fib_recursive(n - 1);
}

fun bench(f, n) {
    var start = clock();
    var result = f(n);
    var time_secs = clock() - start;
    print result;
    print time_secs;
}

var n = 28;

print "iterative:";
bench(fib_iterative, n);
print "\nrecursive:";
bench(fib_recursive, n);
```
Output:
```
iterative:
317811
0.000029325485229492188

recursive:
317811
0.564891815185546
```

## Development 
Run the following command in the root of the project to start the Dioxus dev server:

```bash
dx serve --bin www --hot-reload
```

- Open the browser to http://localhost:8080