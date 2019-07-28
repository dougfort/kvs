use clap::{App, Arg};
use failure::Error;
use kvs::{KvStore, KvsEngine};
use std::env;
use std::process;

fn main() -> Result<(), Error> {
    let matches = App::new(env::var("CARGO_PKG_NAME").unwrap())
        .version(&*env::var("CARGO_PKG_VERSION").unwrap())
        .author(&*env::var("CARGO_PKG_AUTHORS").unwrap())
        .about(&*env::var("CARGO_PKG_DESCRIPTION").unwrap())
        .arg(
            Arg::with_name("action")
                .value_name("ACTION")
                .takes_value(true),
        )
        .arg(Arg::with_name("key").value_name("KEY").takes_value(true))
        .arg(
            Arg::with_name("value")
                .value_name("VALUE")
                .takes_value(true),
        )
        .get_matches();

    let cwd = env::current_dir()?;
    let store = KvStore::open(&cwd)?;

    if let Some(action) = matches.value_of("action") {
        if let Some(key) = matches.value_of("key") {
            match action {
                "get" => {
                    if matches.value_of("value").is_some() {
                        eprintln!("too many arguments");
                        process::exit(2);
                    } else {
                        match store.get(key.to_owned())? {
                            None => println!("Key not found"),
                            Some(value) => println!("{}", value),
                        }
                    }
                }
                "set" => {
                    if let Some(value) = matches.value_of("value") {
                        store.set(key.to_string(), value.to_string())?;
                    } else {
                        eprintln!("you must specify a value");
                        process::exit(2);
                    }
                }
                "rm" => {
                    store.remove(key.to_string())?;
                }
                _ => {
                    eprintln!("unknown action: '{}'", action);
                    process::exit(2);
                }
            }
        } else {
            eprintln!("you must specify a key");
            process::exit(2);
        }
    } else {
        eprintln!("you must specify an action");
        process::exit(2);
    }

    Ok(())
}
