# floof

## Description:

Developed and tested on Windows 10, VSCode, powershell using 

- stable-x86_64-pc-windows-msvc
- cargo 1.55.0 (32da73ab1 2021-08-23)
- rustc 1.55.0 (c8dfcfe04 2021-09-06)

## Usage:

at top-level:

```
cargo run -- path/to/tx.csv
```

For example, using one of the sample test csvs:

```
cargo run -- ./tests/test2.csv
```

There is a verbose option that logs warnings:

```
cargo run -- path/to/tx.csv --verbose
```

## Tests:

```
cargo t
```
