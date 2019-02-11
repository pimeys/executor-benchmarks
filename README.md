# Executor Benchmarks

This repository contains benchmarks for all kinds of futures executors in Rust.

The goal is to:

* Build a reliable set of benchmarks for optimizing Tokio's executors.
* Learn how the new task system in `futures-0.3` affects performance.
* Identify strong and weak points of executors and compare them.

## Examples

Run all benchmarks and save them into `out`:

```console
$ cargo bench | tee out
    Finished release [optimized] target(s) in 0.08s
     Running target/release/deps/bench-0372ae0658dd822f

running 16 tests
test juliex::notify_self                ... bench:   1,421,991 ns/iter (+/- 212,504)
test juliex::poll_reactor               ... bench:  47,680,074 ns/iter (+/- 16,383,224)
test juliex::smoke                      ... bench:       3,971 ns/iter (+/- 758)
test juliex::spawn_many                 ... bench:  21,137,293 ns/iter (+/- 7,435,654)
test tokio::notify_self                 ... bench:   5,319,144 ns/iter (+/- 467,367)
test tokio::poll_reactor                ... bench:  21,596,372 ns/iter (+/- 2,553,090)
test tokio::smoke                       ... bench:     282,052 ns/iter (+/- 64,698)
test tokio::spawn_many                  ... bench:  14,445,574 ns/iter (+/- 767,630)
test tokio_current_thread::notify_self  ... bench:  11,809,885 ns/iter (+/- 300,470)
test tokio_current_thread::poll_reactor ... bench:  35,196,354 ns/iter (+/- 747,981)
test tokio_current_thread::smoke        ... bench:       9,439 ns/iter (+/- 316)
test tokio_current_thread::spawn_many   ... bench:   8,742,460 ns/iter (+/- 454,436)
test tokio_io_pool::notify_self         ... bench:   6,039,105 ns/iter (+/- 755,075)
test tokio_io_pool::poll_reactor        ... bench:  15,023,109 ns/iter (+/- 1,627,964)
test tokio_io_pool::smoke               ... bench:     281,672 ns/iter (+/- 38,259)
test tokio_io_pool::spawn_many          ... bench:  51,466,931 ns/iter (+/- 8,450,450)

test result: ok. 0 passed; 0 failed; 0 ignored; 16 measured; 0 filtered out
```

Compare `tokio` and `tokio-io-pool` using [`cargo-benchcmp`]:

```console
$ cargo benchcmp tokio:: tokio_io_pool:: out
 name          tokio:: ns/iter  tokio_io_pool:: ns/iter  diff ns/iter   diff %  speedup
 notify_self   5,319,144        6,039,105                     719,961   13.54%   x 0.88
 poll_reactor  21,596,372       15,023,109                 -6,573,263  -30.44%   x 1.44
 smoke         282,052          281,672                          -380   -0.13%   x 1.00
 spawn_many    14,445,574       51,466,931                 37,021,357  256.28%   x 0.28
```

Only run `tokio` and `juliex` benchmarks and compare them:

```console
$ A=tokio; B=juliex; > out && cargo bench $A:: | tee -a out && cargo bench $B:: | tee -a out
    Finished release [optimized] target(s) in 0.08s
     Running target/release/deps/bench-eb772ba4902fc8d0

running 4 tests
test tokio::notify_self                 ... bench:   5,330,100 ns/iter (+/- 1,070,439)
test tokio::poll_reactor                ... bench:  21,310,566 ns/iter (+/- 2,490,406)
test tokio::smoke                       ... bench:     283,505 ns/iter (+/- 81,057)
test tokio::spawn_many                  ... bench:  14,442,441 ns/iter (+/- 1,010,884)

test result: ok. 0 passed; 0 failed; 0 ignored; 4 measured; 12 filtered out

    Finished release [optimized] target(s) in 0.07s
     Running target/release/deps/bench-eb772ba4902fc8d0

running 4 tests
test juliex::notify_self                ... bench:   1,426,900 ns/iter (+/- 352,055)
test juliex::poll_reactor               ... bench:  50,735,059 ns/iter (+/- 18,407,718)
test juliex::smoke                      ... bench:       4,065 ns/iter (+/- 826)
test juliex::spawn_many                 ... bench:  19,773,953 ns/iter (+/- 5,982,616)

test result: ok. 0 passed; 0 failed; 0 ignored; 4 measured; 12 filtered out

$ cargo benchcmp $A:: $B:: out
 name          tokio:: ns/iter  juliex:: ns/iter  diff ns/iter   diff %  speedup
 notify_self   5,330,100        1,426,900           -3,903,200  -73.23%   x 3.74
 poll_reactor  21,310,566       50,735,059          29,424,493  138.07%   x 0.42
 smoke         283,505          4,065                 -279,440  -98.57%  x 69.74
 spawn_many    14,442,441       19,773,953           5,331,512   36.92%   x 0.73
```

[`cargo-benchcmp`]: https://github.com/BurntSushi/cargo-benchcmp

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
