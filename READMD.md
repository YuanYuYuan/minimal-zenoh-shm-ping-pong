## An example for measuring the latency of Zenoh with Low-latnecy Transport + Unixpipe Link + SHM Buffer

### How to build

```bash
cargo build --release --bins
```

### How to run

- Terminal 1
```bash
taskset -c 0,2 ./target/release/z_pong ./config/pong.json5
```

- Terminal 2
```bash
taskset -c 1,3 ./target/release/z_ping 33554432 ./config/ping.json5
```

Example output
```log
...
33554432 bytes: seq=93 rtt=27µs lat=13µs
33554432 bytes: seq=94 rtt=30µs lat=15µs
33554432 bytes: seq=95 rtt=28µs lat=14µs
33554432 bytes: seq=96 rtt=27µs lat=13µs
33554432 bytes: seq=97 rtt=28µs lat=14µs
33554432 bytes: seq=98 rtt=27µs lat=13µs
33554432 bytes: seq=99 rtt=27µs lat=13µs
```

### Size Optimization

We could optimize the size of the binaries by fine tuning the building profile in _Cargo.toml_.

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

```txt
───┬─────────────────────────┬──────┬─────────
 # │          name           │ type │  size
───┼─────────────────────────┼──────┼─────────
 0 │ target/release/z_ping   │ file │ 2.6 MiB
 1 │ target/release/z_pong   │ file │ 2.6 MiB
───┴─────────────────────────┴──────┴─────────
```

Without the modfication, the size of a vanilla release build could be larger.

```txt
───┬─────────────────────────┬──────┬─────────
 # │          name           │ type │  size
───┼─────────────────────────┼──────┼─────────
 0 │ target/release/z_ping   │ file │ 9.3 MiB
 1 │ target/release/z_pong   │ file │ 9.3 MiB
───┴─────────────────────────┴──────┴─────────
```
