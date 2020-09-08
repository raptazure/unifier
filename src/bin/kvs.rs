use clap::AppSettings;
use kvs::{KvStore, KvsError, Result};
use std::{env::current_dir, process::exit};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs", 
            global_settings = &
        [AppSettings::DisableHelpSubcommand, AppSettings::VersionlessSubcommands])]

struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "get", about = "Get the string value of a given string key")]
    Get {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
    },
    #[structopt(name = "set", about = "Set the value of a string key to a string")]
    Set {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
        #[structopt(name = "VALUE", help = "The string value of the key")]
        value: String,
    },
    #[structopt(name = "rm", about = "Remove a given string key")]
    Remove {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    match opt.command {
        Command::Get { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(key.to_string())? {
                println!("{}", value);
            } else {
                println!("Ket not found.");
            }
        }
        Command::Set { key, value } => {
            let mut store = KvStore::open(current_dir()?)?;
            store.set(key.to_string(), value.to_string())?;
        }
        Command::Remove { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(key.to_string()) {
                Ok(()) => {}
                Err(KvsError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}
