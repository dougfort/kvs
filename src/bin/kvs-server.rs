use clap::{App, Arg};
use failure::Error;
use kvs::{read_message, write_message, Action, Message};
use kvs::{KvStore, KvsEngine};
use slog::{debug, error, info, o, Drain, Logger};
use slog_async::Async;
use std::env;
use std::net::{TcpListener, TcpStream};

fn main() -> Result<(), Error> {
    let drain = slog_json::Json::new(std::io::stdout())
        .add_default_keys()
        .build()
        .fuse();

    let async_drain = Async::new(drain).build().fuse();
    let server_info = format!("v{}", env!("CARGO_PKG_VERSION"));
    let root_log_context = o!("KVS Server" => server_info);
    let root_logger = Logger::root(async_drain, root_log_context);

    let matches = App::new(env::var("CARGO_PKG_NAME").unwrap())
        .version(&*env::var("CARGO_PKG_VERSION").unwrap())
        .author(&*env::var("CARGO_PKG_AUTHORS").unwrap())
        .about(&*env::var("CARGO_PKG_DESCRIPTION").unwrap())
        .arg(
            Arg::with_name("addr")
                .value_name("IP_PORT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("engine")
                .value_name("ENGINE_NAME")
                .takes_value(true),
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");
    let engine = matches.value_of("engine").unwrap_or_default();
    info!(root_logger, "addr = {}; engine = {}", addr, engine);

    let cwd = env::current_dir()?;
    let store = KvStore::open(&cwd)?;

    let listener = TcpListener::bind(addr)?;
    // accept connections and process them serially
    for stream in listener.incoming() {
        let stream = stream?;
        let peer_addr = stream.peer_addr()?;
        debug!(root_logger, "connection"; "addr" => peer_addr);
        handle_client(stream, root_logger.clone(), store.clone());
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, logger: Logger, store: impl KvsEngine) {
    match read_message(&mut stream) {
        Ok(msg) => {
            let reply_message = if let Message::Command(cmd) = msg {
                match cmd.action {
                    Action::Get => match store.get(cmd.key) {
                        Ok(result) => match result {
                            None => Message::Error("Key not found".to_owned()),
                            Some(value) => Message::String(value),
                        },
                        Err(err) => Message::Error(format!("{}", err)),
                    },
                    Action::Set => match store.set(cmd.key, cmd.value) {
                        Ok(_) => Message::String("ok".to_owned()),
                        Err(err) => Message::Error(format!("{}", err)),
                    },
                    Action::Remove => match store.remove(cmd.key) {
                        Ok(_) => Message::String("ok".to_owned()),
                        Err(err) => Message::Error(format!("{}", err)),
                    },
                    _ => Message::Error("invalid Action".to_string()),
                }
            } else {
                Message::Error("Invalid message".to_string())
            };
            write_message(&mut stream, &reply_message).expect("write_message failed");
        }
        Err(err) => error!(logger, "read_message: {}", err),
    }
    drop(stream);
}
