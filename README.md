
<p align="center">
    <img width="180" alt="Logo" src="extra/logo.png">
</p>
<p align="center">
    A multi-threaded, persistent key/value store server and client with networking over a custom protocol.
</p>

## Quick start

- Build: `cargo build`
- Run server: `unifier-server -h`
  ```
  unifier-server 0.1.0

  USAGE:
    unifier-server [OPTIONS]

  FLAGS:
      -h, --help       Prints help information
      -V, --version    Prints version information

  OPTIONS:
          --addr <IP:PORT>          Sets the listening address [default: 127.0.0.1:4000]
          --engine <ENGINE-NAME>    Sets the storage engine [possible values: kvs, sled]
  ```
  Note: If `--engine` is specified, then `ENGINE-NAME` must be either "kvs", in which
  case the built-in engine is used, or "sled", in which case sled is used. If
  this is the first run (there is no data previously persisted) then the default
  value is "kvs"; if there is previously persisted data then the default is the
  engine already in use. If data was previously persisted with a different
  engine than selected, it will print an error.
- Run client: `unifier-client -h`
  ```
  unifier-client 0.1.0

	USAGE:
    unifier-client <SUBCOMMAND>

  FLAGS:
      -h, --help       Prints help information
      -V, --version    Prints version information

  SUBCOMMANDS:
      get    Get the string value of a given string key
      rm     Remove a given string key
      set    Set the value of a string key to a string
  ```

## Why unifier (unity.kv)?

- Multi-threaded: many threads are created within a process, executing independently but concurrently sharing process resources to finish tasks in a much faster way. This efficiency comes from the unity of threads.
- Traits for intergration:
  - Two key-value store engines. `KvsEngine` trait defines the storage interface. `KvStore` implements `KvsEngine` for the `kvs` storage engine and `SledKvsEngine` implements `KvsEngine` for the `sled` storage engine.  
  - Three threadpool implementations. `ThreadPool` trait contains methods that create a new thread pool to spawn the specified number of threads, and that spawn a function into the threadpool. `NaiveThreadPool` implements `ThreadPool` and spawns a new thread every time the `spawn` method is called. `RayonThreadPool` implements `ThreadPool` using a data parallelism library called [rayon](https://github.com/rayon-rs/rayon). And `SharedQueueThreadPool` implements `ThreadPool` using a shared queue.
  - Different kinds of engines and threadpools to choose is the unity of implementations.
- Built on top of open-source projects and online tutorials, this is the unity of crates and experiences.
- I have been playing _Assassin's Creed Unity_ recently.  :)

## Benchmark

Compared with [sled](https://github.com/spacejam/sled) (a concurrent embedded key-value database), `kvs` engine takes samller space and offers faster speed. The benchmark results are as follows:

<br/>

<img src="extra/line.png" alt="line">
<img src="extra/kvs_8.png" alt="kvs_8">
<img src="extra/sled_8.png" alt="sled_8">
<img src="extra/kvs_16.png" alt="kvs_16">
<img src="extra/sled_16.png" alt="sled_16">


## Testing
`cargo test --test cli`
- [x] server_cli_version
- [x] client_cli_version
- [x] client_cli_no_args
- [x] client_cli_invalid_subcommand
- [x] test client_cli_invalid_get
- [x] client_cli_invalid_rm
- [x] client_cli_invalid_set
- [x] cli_log_configuration
- [x] cli_wrong_engine
- [x] cli_access_server_kvs_engine
- [x] cli_access_server_sled_engine

`cargo test --test kv_store`
- [x] remove_non_existent_key
- [x] remove_key
- [x] get_non_existent_value
- [x] get_stored_value
- [x] overwrite_value
- [x] concurrent_get
- [x] concurrent_set
- [x] compaction

`cargo test --test thread_pool`
- [x] naive_thread_pool_spawn_counter
- [x] shared_queue_thread_pool_spawn_counter
- [x] rayon_thread_pool_spawn_counter
- [x] shared_queue_thread_pool_panic_task

<br />

> Thanks to https://github.com/pingcap/talent-plan
