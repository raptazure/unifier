
<p align="center">
    <img width="180" alt="Logo" src="extra/logo.png">
</p>
<p align="center">
    A key/value server and client (communicate with a custom networking protocol)
</p>

## Quick start

- run: `cargo run`
- build: `cargo build`
- test: `cargo test`
- benchmark: `cargo bench`

## Project spec

The `kvs-server` executable supports the following command line arguments:

- `kvs-server [--addr IP-PORT] [--engine ENGINE-NAME]`

  Start the server and begin listening for incoming connections. `--addr`
  accepts an IP address, either v4 or v6, and a port number, with the format
  `IP:PORT`. If `--addr` is not specified then listen on `127.0.0.1:4000`.

  If `--engine` is specified, then `ENGINE-NAME` must be either "kvs", in which
  case the built-in engine is used, or "sled", in which case sled is used. If
  this is the first run (there is no data previously persisted) then the default
  value is "kvs"; if there is previously persisted data then the default is the
  engine already in use. If data was previously persisted with a different
  engine than selected, print an error and exit with a non-zero exit code.

  Print an error and return a non-zero exit code on failure to bind a socket, if
  `ENGINE-NAME` is invalid, if `IP-PORT` does not parse as an address.

- `kvs-server -V`

  Print the version.

The `kvs-client` executable supports the following command line arguments:

- `kvs-client set <KEY> <VALUE> [--addr IP-PORT]`

  Set the value of a string key to a string.

  `--addr` accepts an IP address, either v4 or v6, and a port number, with the
  format `IP:PORT`. If `--addr` is not specified then connect on
  `127.0.0.1:4000`.

  Print an error and return a non-zero exit code on server error,
  or if `IP-PORT` does not parse as an address.

- `kvs-client get <KEY> [--addr IP-PORT]`

  Get the string value of a given string key.

  `--addr` accepts an IP address, either v4 or v6, and a port number, with the
  format `IP:PORT`. If `--addr` is not specified then connect on
  `127.0.0.1:4000`.

  Print an error and return a non-zero exit code on server error,
  or if `IP-PORT` does not parse as an address.

- `kvs-client rm <KEY> [--addr IP-PORT]`

  Remove a given string key.

  `--addr` accepts an IP address, either v4 or v6, and a port number, with the
  format `IP:PORT`. If `--addr` is not specified then connect on
  `127.0.0.1:4000`.

  Print an error and return a non-zero exit code on server error,
  or if `IP-PORT` does not parse as an address. A "key not found" is also
  treated as an error in the "rm" command.

- `kvs-client -V`

  Print the version.

The `kvs` library contains four types:

- `KvsClient` - implements the functionality required for `kvs-client` to speak
  to `kvs-server`
- `KvsServer` - implements the functionality to serve responses to `kvs-client`
  from `kvs-server`
- `KvsEngine` trait - defines the storage interface called by `KvsServer`
- `KvStore` - implements by hand the `KvsEngine` trait
- `SledKvsEngine` - implements `KvsEngine` for the [`sled`] storage engine.

[`sled`]: https://github.com/spacejam/sled

The `KvsEngine` trait supports the following methods:

- `KvsEngine::set(&mut self, key: String, value: String) -> Result<()>`

  Set the value of a string key to a string.

  Return an error if the value is not written successfully.

- `KvsEngine::get(&mut self, key: String) -> Result<Option<String>>`

  Get the string value of a string key.
  If the key does not exist, return `None`.

  Return an error if the value is not read successfully.

- `KvsEngine::remove(&mut self, key: String) -> Result<()>`

  Remove a given string key.

  Return an error if the key does not exit or value is not read successfully.

When setting a key to a value, `KvStore` writes the `set` command to disk in
a sequential log. When removing a key, `KvStore` writes the `rm` command to
the log. On startup, the commands in the log are re-evaluated and the
log pointer (file offset) of the last command to set each key recorded in the
in-memory index.

When retrieving a value for a key with the `get` command, it searches the index,
and if found then loads from the log, and evaluates, the command at the
corresponding log pointer.

When the size of the uncompacted log entries reach a given threshold, `KvStore`
compacts it into a new log, removing redundant entries to reclaim disk space.

## Benchmark

```log
set_bench/kvs           time:   [544.33 us 546.36 us 548.72 us]                          
                        change: [-1.0349% -0.1713% +0.6824%] (p = 0.71 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

set_bench/sled          time:   [362.76 ms 369.02 ms 374.07 ms]                           
                        change: [+106.42% +137.23% +175.84%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low severe
  2 (2.00%) high mild

get_bench/kvs/4         time:   [1.9685 us 1.9766 us 1.9864 us]                             
                        change: [-8.3731% -6.9574% -5.6073%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) high mild
  8 (8.00%) high severe

get_bench/kvs/8         time:   [2.2268 us 2.2303 us 2.2340 us]                             
                        change: [-4.8781% -3.7775% -2.6816%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe

get_bench/sled/4        time:   [588.43 ns 593.85 ns 600.38 ns]                              
                        change: [-4.5363% +0.1171% +4.8562%] (p = 0.97 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

get_bench/sled/8        time:   [662.02 ns 668.51 ns 676.30 ns]                              
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
```

<br />

> Thanks to https://github.com/pingcap/talent-plan
