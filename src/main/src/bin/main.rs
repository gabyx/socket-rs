use anyhow::{Context, Result};
use clap::Parser;
use common::comm;
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

    let mut prev_msg: Option<String> = None;
    for i in 0..10 {
        let ping = format!("ping-{side:?}-{i}");
        let mut s: usize = 0;

        while s != ping.len() {
            info!(&log, "Sending ping."; "i" => i);
            s = sock
                .send(ping.as_bytes())
                .with_context(|| "could not send ping...")?;
        }

        let mut msg: [u8; 10] = [0; 10];
        let mut r: usize = 0;
        while r < ping.len() {
            info!(&log, "Receiving ping."; "i" => i);
            r = sock.recv(&mut msg).unwrap_or_else(|_| {
                warn!(log, "could not receive");
                0
            });
        }

        let m = str::from_utf8(msg.as_slice())
            .with_context(|| "could not convert recv. bytes to utf8")?;

        info!(log, "Received msg {m:?}");
        if let Some(p) = prev_msg
            && p.as_str() >= m
        {
            warn!(log, "Message is not in sequence > {p}.");
        }
        prev_msg = Some(m.to_owned());
    }

    Ok(())
}
