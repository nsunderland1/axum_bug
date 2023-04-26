This repo demonstrates a bug (or more likely just a breaking change) in axum 0.6:

1. By default this package depends on `axum-0.5.17`. If you run `cargo test`, you should see this:

```
running 1 test
test tests::admission_control ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.00s
```

2. Update the dependency in `Cargo.toml` to `axum = "0.6"` and run `cargo test` again. You should see this:

```
running 1 test
test tests::admission_control ... FAILED

failures:

---- tests::admission_control stdout ----
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
thread 'tests::admission_control' panicked at 'assertion failed: cur < CONCURRENCY', src/main.rs:70:17
thread 'tests::admission_control' panicked at 'called `Result::unwrap()` on an `Err` value: JoinError::Panic(Id(1), ...)', src/main.rs:107:41


failures:
    tests::admission_control

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

See `main.rs` for details on the test.