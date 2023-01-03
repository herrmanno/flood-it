# Flood it SAT solver

Solving ``Flood it'' [0] [1] like puzzles via [z3][2].

![logo](logo.png)

## Build

### Requirements

Requires a local installation of [z3][2] to be present.

### Compile

```sh
cargo build --release
```

## Usage

### Find solution w/ n = #clusters
```sh
./target/release/color-flood-rs < input.txt
```

### Find optimal solution by binary search
```sh
./target/release/color-flood-rs min < input.txt
```

### Find optimal solution in fixed range by binary search 
```sh
./target/release/color-flood-rs [lo] [hi] < input.txt
```

### Find solution with fixed number of steps
```sh
./target/release/color-flood-rs [steps] < input.txt
```

## Documentation

Documentation can be build and watched via
```shell
cargo doc --open
```

## Input format

Problem instances must be formated as ASCII text, where each line, separated by a newline character,
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