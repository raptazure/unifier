#![deny(missing_docs)]
//! A simple key/value store.

#[macro_use]
extern crate log;

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Result};
pub use server::KvsServer;

mod client;
mod common;
mod engines;
mod error;
<<<<<<< HEAD
mod server;
=======
mod server;
>>>>>>> 81ce36630f02ba3a18355fd66a5f3bb0a023c87e
