use clap::{App, Arg};
use failure::Error;
use kvs::KvStore;
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
    let mut store = KvStore::open(&cwd)?;

    let action = matches.value_of("action").unwrap_or_default();
    match action {
        "get" => {
            let key = matches.value_of("key").unwrap_or_default();
            let get_result = store.get(key.to_owned())?;
            if get_result.is_none() {
                println!("Key not found");
            };
        }
        "set" => {
            let key = matches.value_of("key").unwrap_or("");
            let value = matches.value_of("value").unwrap_or("");
            store.set(key.to_string(), value.to_string())?;
        }
        "rm" => {
            let key = matches.value_of("rm").unwrap_or("");
            let rm_result = store.remove(key.to_string())?;
            if rm_result.is_none() {
                println!("Key not found");
            };
        }
        _ => {
            eprintln!("unknown action: '{}'", action);
            process::exit(2);
        }
    }

    Ok(())
}
