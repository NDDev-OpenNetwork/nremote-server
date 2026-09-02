mod common;
use common::ArgSpec;
mod relay_server;
use flexi_logger::*;
use hbb_common::{config::RELAY_PORT, ResultType};
use relay_server::*;
mod version;

fn main() -> ResultType<()> {
    let _logger = Logger::try_with_env_or_str("info")?
        .log_to_stdout()
        .format(opt_format)
        .write_mode(WriteMode::Async)
        .start()?;
    static ARGS: &[ArgSpec] = &[
        ArgSpec {
            id: "bind",
            short: Some('b'),
            long: "bind",
            value_name: "IP",
            help: "Sets the IP address to bind to (default: all interfaces)",
        },
        ArgSpec {
            id: "port",
            short: Some('p'),
            long: "port",
            value_name: "NUMBER",
            help: "Sets the listening port (default 21117)",
        },
        ArgSpec {
            id: "key",
            short: Some('k'),
            long: "key",
            value_name: "KEY",
            help: "Only allow the client with the same key",
        },
    ];
    // hbbr used to build its own parser and load `.env` itself, which was the
    // same twenty lines as hbbs with three arguments instead of nine. It does
    // not accept `--config`, and `init_args` simply finds no such argument.
    let matches = common::init_args(ARGS, "hbbr", "nremote relay server");

    let mut port = RELAY_PORT;
    if let Some(v) = common::get_arg_opt("PORT") {
        let v: i32 = v.parse().unwrap_or_default();
        if v > 0 {
            port = v + 1;
        }
    }
    let bind = matches
        .get_one::<String>("bind")
        .cloned()
        .unwrap_or_else(|| common::get_arg("BIND"));
    let bind_addr = common::parse_bind_address(&bind)?;
    let key = matches
        .get_one::<String>("key")
        .cloned()
        .unwrap_or_else(|| common::get_arg("KEY"));
    // Bound to a local rather than passed inline: the old form borrowed a
    // temporary `port.to_string()` inside the call, which lived exactly long
    // enough and read as if it did not.
    let port = matches
        .get_one::<String>("port")
        .cloned()
        .unwrap_or_else(|| port.to_string());
    start_with_bind(bind_addr, &port, &key)?;
    Ok(())
}
