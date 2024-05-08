# Performance

Based on time and resources taken to parse and index docs.gl folder

Run command to run performance:

```bash
cargo build
cargo build --release
time ./target/debug/engine index ./docs.gl
time ./target/release/engine index ./docs.gl
```

<!--  Template:

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 4.14s     | 10.69s      | 97%       | 15.222     |

-->

### V0.0.2-beta

##### debug:

user 8.84s system 93% cpu 13.751 total

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 8.08s     | 8.78s       | 95%       | 17.614     |

##### release:

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 3.93s     | 8.67s       | 93%       | 12.885     |

###### Serve time:

> [!NOTE] > _Serve time is the time taken to search for a term in the index_

- **LATEST** After caching `TF` calculation:

  > | debug | release |
  > | ----- | ------- |
  > | 19ms  | 9ms     |

- After caching `IDf` calculation:

  > | debug | release |
  > | ----- | ------- |
  > | 140ms | 14ms    |

### V0.0.1-beta

##### debug:

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 8.23s     | 10.78s      | 95%       | 19.842     |

##### release:

| User Time | System Time | CPU Usage | Total Time |
| --------- | ----------- | --------- | ---------- |
| 4.14s     | 10.69s      | 97%       | 15.222     |

###### Serve time:

| debug | release |
| ----- | ------- |
| 200ms | 150ms   |
