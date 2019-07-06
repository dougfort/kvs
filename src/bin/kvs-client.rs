use clap::{App, Arg};
use failure::Error;
use std::env;
use std::process;
use std::net::TcpStream;

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
        .arg(
            Arg::with_name("addr")
                .value_name("IP_PORT")
                .takes_value(true),
        )
        .get_matches();


    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");
    if let Ok(_stream) = TcpStream::connect(addr) {
        println!("Connected to the server!");
    } else {
        println!("Couldn't connect to server...");
    }

    if let Some(action) = matches.value_of("action") {
        if let Some(key) = matches.value_of("key") {
            match action {
                "get" => {
                    if let Some(_) = matches.value_of("value") {
                        eprintln!("too many arguments");
                        process::exit(2);
                    } else {
                        println!("get {}", key);
                    }
                }
                "set" => {
                    if let Some(value) = matches.value_of("value") {
                        println!("set {} {}", key, value);
                    } else {
                        eprintln!("you must specify a value");
                        process::exit(2);
                    }
                }
                "rm" => {
                    println!("rm {}", key);
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
