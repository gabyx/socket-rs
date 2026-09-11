use anyhow::{Context, Result, anyhow};
use slog::{Logger, info, warn};
use std::{
    net::{self, Ipv4Addr, SocketAddr, ToSocketAddrs},
    ops::Range,
    random::{Rng, SystemRng},
    str::FromStr,
};

// STUN message header — RFC 5389 §6
//
// Fixed 20 bytes, everything big-endian (network order):
//
//   0                   1                   2                   3
//   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |0 0|     STUN Message Type     |         Message Length        |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |                         Magic Cookie (0x2112A442)             |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  |                                                               |
//  |                     Transaction ID (12 bytes)                 |
//  |                                                               |
//  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[derive(Debug, Copy, Clone)]
struct StunHeader([u8; Self::LEN]);
// {
// msg_type: u16,
// msg_length: u16,
// cookie: u32,
// transaction_id: [u8; 12],
// }

pub const PUBLIC_STUN_SERVER: (&str, u16) = ("stun.l.google.com", 19302);
const MAGIC_COOKIE: u32 = 0x2112_A442;

// The message type is interleaved with class and method:
// `0 0 M11 M10 M9 M8 M7 C1 M6 M5 M4 C0 M3 M2 M1 M0`:
//
//   - class 00 = request,
//           01 = indication,
//           10 = success response,
//           11 = error response
//   - method 0x001 = Binding (the only one we need)
//
//  which gives:
#[repr(u16)]
#[derive(Debug, Copy, Clone)]
enum MsgType {
    BindingRequest = 0x0001,
    BindingResponse = 0x0101,
    BindingResponseError = 0x0111,
}

impl StunHeader {
    pub const LEN: usize = 20;

    const TYPE: Range<usize> = 0..2; // name the offsets once
    const LENGTH: Range<usize> = 2..4;
    const COOKIE: Range<usize> = 4..8;
    const TXID: Range<usize> = 8..20;

    fn set_message_type(&mut self, t: MsgType) -> &mut StunHeader {
        self.0[Self::TYPE].copy_from_slice(&(t as u16).to_be_bytes());
        self
    }

    fn set_length(&mut self, l: u16) -> &mut StunHeader {
        self.0[Self::LENGTH].copy_from_slice(&l.to_be_bytes());
        self
    }

    fn set_magic_cookie(&mut self) -> &mut StunHeader {
        // The cookie is sent in network byte order.
        self.0[Self::COOKIE].copy_from_slice(&MAGIC_COOKIE);
        self
    }

    fn set_transaction_id(&mut self) -> &mut StunHeader {
        let mut rnd = SystemRng;
        let mut id = [0u8; 12];
        rnd.fill_bytes(&mut id);
        self.0[Self::TXID].copy_from_slice(&id);
        self
    }

    fn new() -> StunHeader {
        let mut s = StunHeader([0; Self::LEN]);
        s.set_magic_cookie().set_transaction_id();
        s
    }

    pub fn new_binding_request() -> StunHeader {
        *StunHeader::new()
            .set_length(0)
            .set_message_type(MsgType::BindingRequest)
    }
}

// Sends a STUN message to a public stun server to discover our IP
// address.
pub fn send_stun(log: &Logger, socket: &net::UdpSocket, server: (&str, u16)) -> Result<Ipv4Addr> {
    let server: SocketAddr = server
        .to_socket_addrs()?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| anyhow!("no IPv4 address for stun.l.google.com"))?;

    let b = StunHeader::new_binding_request();

    info!(log, "Sending stun header to {server}:\n   {:02X?}", b.0);
    if let Err(e) = socket.send_to(&b.0, server) {
        warn!(log, "could not send: {e}");
        return Err(e).context("failed to send STUN binding response");
    }

    Ok(Ipv4Addr::from_str("1.1.1.1").unwrap())
}
