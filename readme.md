# CompGraph

Computational graph written in Rust.

## Examples


To create a computational graph such as `x1 + x2 * sin(x2 + pow(x3, 3))`  use supplied `sum, mul, sub, div, sin, cos, pow` functions.

First create a graph input with `create_input(name)` or `create_input_with(name, value)`.

```rust
use compgraph::create_input;

let x1 = create_input("x1");
let x2 = create_input("x2");
let x3 = create_input("x3");
```

Then create graph 

```rust
use compgraph::{mul, sum, sin, pow};

let cgraph = sum(x1.clone(), mul(x2.clone(), sin(sum(x2.clone(), pow(x3, 3)))))
```

Don't forget to set input values before calling `compute()`

```rust
x1.set(10.);
x2.set(20.);
x3.set(30.);


let result = cgraph.compute();
```


## TODO

 - [ ] Add "compound" tests
 - [ ] Add arena-based graph
 - [ ] Add benchmarks
 - [ ] Find a way to reduce "copypaste" code in some of a functions (proc_macro?)
