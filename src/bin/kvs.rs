use clap::{App, Arg};
use kvs::KvStore;
use std::env;
use std::process;

fn main() {
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

    let action = matches.value_of("action").unwrap();
    let mut store = KvStore::new();
    match action {
        "get" => {
            let key = matches.value_of("key").unwrap_or("");
            let result = store.get(key.to_string());
            println!("action = {}, result = {:?}", action, result);
        }
        "set" => {
            let key = matches.value_of("key").unwrap_or("");
            let value = matches.value_of("value").unwrap_or("");
            store.set(key.to_string(), value.to_string());
            println!("action = {}", action);
        }
        "rm" => {
            let key = matches.value_of("rm").unwrap_or("");
            store.remove(key.to_string());
            println!("action = {}", action);
        }
        _ => {
            eprintln!("unknown action: '{}'", action);
            process::exit(2);
        }
    }
}
