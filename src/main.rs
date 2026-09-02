// https://tools.ietf.org/rfc/rfc5128.txt
// https://blog.csdn.net/bytxl/article/details/44344855

use flexi_logger::*;
use hbb_common::{bail, config::RENDEZVOUS_PORT, ResultType};
use hbbs::{common::*, *};

const RMEM: usize = 0;

fn main() -> ResultType<()> {
    let _logger = Logger::try_with_env_or_str("info")?
        .log_to_stdout()
        .format(opt_format)
        .write_mode(WriteMode::Async)
        .start()?;
    // `NUMBER(default=…)` in the old usage strings was never a clap default:
    // in clap 2's DSL the bracketed token is only the value name shown in
    // help, and the real defaults are supplied by `get_arg_or` below. Writing
    // them as defaults here would change behaviour, so they stay in the help
    // text where they always were.
    static ARGS: &[ArgSpec] = &[
        ArgSpec { id: "config", short: Some('c'), long: "config", value_name: "FILE",
                  help: "Sets a custom config file" },
        ArgSpec { id: "bind", short: Some('b'), long: "bind", value_name: "IP",
                  help: "Sets the IP address to bind to (default: all interfaces)" },
        ArgSpec { id: "port", short: Some('p'), long: "port", value_name: "NUMBER",
                  help: "Sets the listening port (default 21116)" },
        ArgSpec { id: "serial", short: Some('s'), long: "serial", value_name: "NUMBER",
                  help: "[DEPRECATED] Sets configure update serial number (default 0)" },
        ArgSpec { id: "rendezvous-servers", short: Some('R'), long: "rendezvous-servers",
                  value_name: "HOSTS",
                  help: "[DEPRECATED] Sets rendezvous servers, separated by comma" },
        ArgSpec { id: "relay-servers", short: Some('r'), long: "relay-servers",
                  value_name: "HOST",
                  help: "Sets the default relay servers, separated by comma" },
        ArgSpec { id: "rmem", short: Some('M'), long: "rmem", value_name: "NUMBER",
                  help: "Sets UDP recv buffer size in bytes (default 0). Raise the system limit first: sudo sysctl -w net.core.rmem_max=52428800" },
        ArgSpec { id: "mask", short: None, long: "mask", value_name: "MASK",
                  help: "[DEPRECATED] Determine if the connection comes from LAN, e.g. 192.168.0.0/16" },
        ArgSpec { id: "key", short: Some('k'), long: "key", value_name: "KEY",
                  help: "Only allow the client with the same key" },
    ];
    init_args(ARGS, "hbbs", "nremote ID/rendezvous server");
    let port = get_arg_or("port", RENDEZVOUS_PORT.to_string()).parse::<i32>()?;
    if port < 3 {
        bail!("Invalid port");
    }
    let bind_addr = parse_bind_address(&get_arg("bind"))?;
    let rmem = get_arg("rmem").parse::<usize>().unwrap_or(RMEM);
    let serial: i32 = get_arg("serial").parse().unwrap_or(0);
    RendezvousServer::start_with_bind(
        bind_addr,
        port,
        serial,
        &get_arg_or("key", "-".to_owned()),
        rmem,
    )?;
    Ok(())
}
