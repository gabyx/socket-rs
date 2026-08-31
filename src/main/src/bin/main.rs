use anyhow::{Context, Result};
use clap::Parser;
use common::comm;
use slog::{Drain, Logger, info, o};
use std::net;

/// A simple CLI that says hello.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet.
    #[clap(short, long, default_value = "a")]
    side: comm::Side,
}

/// Build a simple terminal logger: human-readable, timestamped, async.
fn build_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    Logger::root(drain, o!())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log = build_logger();

    info!(log, "Starting up"; "side" => ?args.side);

    let socket = start_socket(&log, args.side)?;

    Ok(())
}

fn start_socket(log: &Logger, side: comm::Side) -> Result<net::UdpSocket> {
    let address = format!("127.0.0.1:{}", side.port());
    info!(log, "Binding UDP socket"; "address" => &address);

    net::UdpSocket::bind(&address).with_context(|| format!("Cannot bind socket on {address}"))
}
