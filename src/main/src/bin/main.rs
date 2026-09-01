use anyhow::{Context, Result};
use clap::Parser;
use common::comm;
use serde::{Deserialize, Serialize};
use slog::{Drain, Logger, info, o, warn};
use std::{
    net,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// A simple CLI that says hello.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet.
    #[clap(short, long, default_value = "a")]
    side: comm::Side,
}

/// One datagram exchanged between the two peers: a ping carrying its sequence
/// number so the receiver can detect loss and reordering on the wire.
#[derive(Debug, Serialize, Deserialize)]
struct Ping {
    side: comm::Side,
    seq: u32,
}

/// Build a simple terminal logger: human-readable, timestamped, async.
fn build_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain)
        .chan_size(10000)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();

    Logger::root(drain, o!())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log = build_logger();

    info!(log, "Starting up"; "side" => ?args.side);

    let socket = start_socket(&log, args.side.other())?;

    ping_pong(&log, args.side, &socket)?;

    Ok(())
}

fn start_socket(log: &Logger, side: comm::Side) -> Result<net::UdpSocket> {
    let our_addr = format!("127.0.0.1:{}", side.port());
    info!(log, "Binding UDP socket"; "address" => &our_addr);

    let socket = net::UdpSocket::bind(&our_addr)
        .with_context(|| format!("Cannot bind socket on {our_addr}"))?;

    let other_addr = format!("127.0.0.1:{}", side.other().port());
    socket
        .connect(other_addr.clone())
        .with_context(|| format!("could not connect to socket {other_addr}"))?;

    Ok(socket)
}

fn wait_for_sync_point(log: &Logger) {
    let time_frame = 10.0;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("could not get time")
        .as_secs_f64();

    let rest = time_frame - (now.rem_euclid(time_frame));

    info!(log, "Waiting {rest} secs to next sync point.");

    sleep(Duration::from_secs_f64(rest));
    info!(log, "Starting");
}

#[allow(clippy::similar_names)]
fn ping_pong(log: &Logger, side: comm::Side, sock: &net::UdpSocket) -> Result<()> {
    wait_for_sync_point(log);

    let mut data = vec![];

    for i in 0..1000 {
        let ping = Ping { seq: i, side };
        let mut s: usize = 0;

        data.clear();
        data = postcard::to_extend(&ping, data).context("could not serialize")?;
        while s != data.len() {
            info!(&log, "Sending ping."; "i" => i);
            s = sock.send(&data).with_context(|| "could not send ping...")?;
        }

        info!(&log, "Receiving ping: {i}");
        if let r = sock.recv(&mut data).unwrap_or_else(|e| {
            warn!(log, "could not receive: {e}");
            0
        }) && r == 0
        {
            warn!(log, "received 0 bytes");
            continue;
        }

        let recv = match postcard::from_bytes::<Ping>(&data) {
            Ok(p) => p,
            Err(e) => {
                warn!(log, "could not decode msg: {e}");
                continue;
            }
        };

        info!(log, "Received: {recv:?}");
        if recv.seq != i {
            warn!(log, "Message is not in sequence {} != {i}.", recv.seq);
        }
    }

    Ok(())
}
