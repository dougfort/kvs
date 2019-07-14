use clap::{App, Arg};
use failure::Error;
use kvs::{Action, Command, Message};
use std::env;
use std::net::TcpStream;
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
        .arg(
            Arg::with_name("addr")
                .value_name("IP_PORT")
                .takes_value(true),
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");
    let mut stream = TcpStream::connect(addr)?;

    if let Some(action) = matches.value_of("action") {
        if let Some(key) = matches.value_of("key") {
            match action {
                "get" => {
                    if matches.value_of("value").is_some() {
                        eprintln!("too many arguments");
                        process::exit(2);
                    } else {
                        println!("get {}", key);
                        let cmd = Command {
                            action: Action::Get,
                            key: key.to_owned(),
                            value: "".to_string(),
                        };
                        let msg = Message::Command(cmd);
                        kvs::write_message(&mut stream, &msg)?;
                        match kvs::read_message(&mut stream)? {
                            Message::String(value) => println!("value = {}", value),
                            Message::Error(err) => {
                                eprintln!("kvs error {}", err);
                                process::exit(2);
                            }
                            _ => {
                                eprintln!("invalid reply");
                                process::exit(2);
                            }
                        }
                    }
                }
                "set" => {
                    if let Some(value) = matches.value_of("value") {
                        println!("set {} {}", key, value);
                        let cmd = Command {
                            action: Action::Set,
                            key: key.to_owned(),
                            value: value.to_owned(),
                        };
                        let msg = Message::Command(cmd);
                        kvs::write_message(&mut stream, &msg)?;
                        match kvs::read_message(&mut stream)? {
                            Message::String(value) => println!("value = {}", value),
                            Message::Error(err) => {
                                eprintln!("kvs error {}", err);
                                process::exit(2);
                            }
                            _ => {
                                eprintln!("invalid reply");
                                process::exit(2);
                            }
                        }
                    } else {
                        eprintln!("you must specify a value");
                        process::exit(2);
                    }
                }
                "rm" => {
                    println!("rm {}", key);
                    let cmd = Command {
                        action: Action::Remove,
                        key: key.to_owned(),
                        value: "".to_owned(),
                    };
                    let msg = Message::Command(cmd);
                    kvs::write_message(&mut stream, &msg)?;
                    match kvs::read_message(&mut stream)? {
                        Message::String(value) => println!("value = {}", value),
                        Message::Error(err) => {
                            eprintln!("kvs error {}", err);
                            process::exit(2);
                        }
                        _ => {
                            eprintln!("invalid reply");
                            process::exit(2);
                        }
                    }
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
