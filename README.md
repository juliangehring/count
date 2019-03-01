# Rusty counting

## Installation

Build it yourself with cargo:

```sh
cargo install --git https://github.com/juliangehring/count
```


## Benchmarking

An anecdotal benchmark: Count the top-level domains of the Alexa top 1 million websites. Measured with [hyperfine](https://github.com/sharkdp/hyperfine).

```sh
make benchmark
```

| Tool                  | Time [ms] |
| --------------------- | --------: |
| rusty count (LTO)     |       124 |
| rusty count (default) |       150 |
| awk                   |       207 |
| unix tools            |      4101 |
