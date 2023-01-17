# Flood it SAT solver

Solving [Flood it][0] like puzzles via [z3][2].

![logo](logo.png)

## Build

### Requirements

Requires a local installation of [z3][2] to be present.

### Compile

```sh
cargo build --release
```

The final binary can be found at `./target/{release,debug}/color-flood-rs`.

## Run

```sh
Usage: color-flood-rs [OPTIONS] <COMMAND>

Commands:
  opt     Use z3 optimizer to find minimal solution
  min     Find minimal solution by binary search in reasonable bounds
  search  Find minimal solution by binary search in bounds
  exact   Find solution with exact size
  solve   Find solution with reasonable large size
  help    Print this message or the help of the given subcommand(s)

Options:
      --print-asserts  Print assert in SMT-LIB format
      --dry-run        Only create asserts but don't solve
  -h, --help           Print help information
  -V, --version        Print version information
```

To show further information about the different modes and their options

```sh
color-floor-rs <COMMAND> --help
```

can be used.

## Documentation

Solving is achieved through multiple SMT formulas over the theory of linear integer
arithmetic.  
Construction of formulas, as well as dispatching them to z3 is done at
[./src/solver.rs](./src/solver.rs). Note that [z3][z3] uses [z3-sys][z3-sys], which utilizes z3
as a dynamic library. Therefore, no additional process is created when dispatching a formula to
the solver.

Dependent on the run mode, the following number of assertions will be made:
- `2*C`: every color must me in valid range
- `C`: every two consecutive colors must be different 
- `N`: every cluster must be flooded at last
- `C`: the start cluster must be flooded at all times
- `N`: no cluster other than the start cluster must be flooded at the start
- `N*C`: Every flooded cluster implies, that it is also flooded at t+1
- `N*C`: Every unflooded cluster must be flooded if a neighbour is flooded and its color was chosen
- `N*C`: No unflooded cluster must be flooded if no neighbour is flooded or its color was not chosen
- If running in optimization mode:
  - `1`: the number of points in time where all clusters are flooded gets maximized, else
  - `1`: at least one cluster must not be flooded at t_max-1
where `C` is the solution length (= number of colors) and `N` is the number of clusters, which
has an upper bound equal to the number of tiles in the problem

This sums up to `O(N*C)` assertions being made.

### Runtimes

The following runtimes are measured against one instance of each size each, which was extracted from
[this website][0]. This times should be taken with caution because show a narrow view of real world
performance. In general it advisable to manually give an upper bound when possible to improve search
time.

| Problem size | `solve` mode | `min` mode | `opt` mode |
|-------------:|-------------:|-----------:|-----------:|
|14x14         |0.4s          |0.6s        |0.7s        |
|18x18         |5.5s          |16s         |10s         |
|20x20         |13s           |2h          |>3h         |
|-------------:|-------------:|-----------:|-----------:|

Further documentation can be build and viewed via
```shell
cargo doc --open
```

## Input format

Problem instances must be formatted as ASCII text, where each line, separated by a newline character,
denotes a row of the problem's grid and every character must be in the ASCII range [48,57].
Colors must be encoded as gapless sequence of ASCII chars starting from 48, such that `n` colors
are decoded by ASCII chars [48,48 + n - 1] respectively.

Example of problem w/ size 6 x 6 using three colors:

```shell
011101
011012
021101
100020
000112
110220
```

### Example instances

Some example instances can be found at [instances](./instances/).

[0]: https://unixpapa.com/floodit
[1]: https://www.janko.at/Spiele/Farbflutung
[2]: https://github.com/Z3Prover/z3
[z3]: https://github.com/prove-rs/z3.rs/tree/master/z3
[z3-sys]: https://github.com/prove-rs/z3.rs/tree/master/z3-sys