use clap::AppSettings;
use std::process::exit;
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

fn main() {
    let opt = Opt::from_args();
    match opt.command {
        Command::Get { key } => {
            eprintln!("unimplemented");
            eprintln!("{}", key);
            exit(1);
        }
        Command::Set { key, value } => {
            eprintln!("unimplemented");
            eprintln!("{}, {}", key, value);
            exit(1);
        }
        Command::Remove { key } => {
            eprintln!("unimplemented");
            eprintln!("{}", key);
            exit(1);
        }
    }
}
