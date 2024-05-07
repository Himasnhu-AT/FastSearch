# Performance

Based on time and resources taken to parse and index docs.gl folder

Run command to run performance:

```bash
cargo build
cargo build --release
time ./target/debug/engine index ./docs.gl
time ./target/release/engine index ./docs.gl
```

### V0.0.1-beta

<!--  Template:

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 4.14s     | 10.69s      | 97%       | 15.222     |

-->

##### debug:

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 8.23s     | 10.78s      | 95%       | 19.842     |

##### release:

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 4.14s     | 10.69s      | 97%       | 15.222     |
