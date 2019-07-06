use clap::{App, Arg};
use failure::Error;
use std::env;
use slog::{Drain, Logger, o, info};
use slog_async::Async;

fn main() -> Result<(), Error> {
    let drain = slog_json::Json::new(std::io::stdout()).add_default_keys()
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

    Ok(())
}
