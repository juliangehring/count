# Rusty counting

A toy project for writing a fast and convenient alternative to

```sh
sort | uniq -c | sort -k1 -rn
```


## Installation

Install it yourself with cargo:

```sh
cargo install --git https://github.com/juliangehring/count
```


## Benchmarking

An anecdotal benchmark: Count the top-level domains of the Alexa top 1 million websites. Measured with [hyperfine](https://github.com/sharkdp/hyperfine).

```sh
make benchmark
```

| Tool             | Time [ms] |
| ---------------- | --------: |
| rusty count (u8) |        30 |
| awk              |       195 |
| unix tools       |      3842 |
