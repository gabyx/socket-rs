use anyhow::{Context, Result, anyhow};
use slog::{Logger, info, warn};
use std::{
    net::{self, Ipv4Addr, SocketAddr, ToSocketAddrs},
    ops::Range,
    random::{Rng, SystemRng},
    result::Result as StdResult,
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
struct StunMessage<'a>(&'a [u8]);

pub const PUBLIC_STUN_SERVER: (&str, u16) = ("stun.l.google.com", 19302);
const MAGIC_COOKIE: u32 = 0x2112_A442;

// The message type is interleaved with class and method:
// u16: `0 0 M11 M10 M9 M8 M7 C1 M6 M5 M4 C0 M3 M2 M1 M0`:
//
//   - class  0x00  = request,
//            0x01  = indication,
//            0x10  = success response,
//            0x11  = error response
//   - method 0x001 = Binding (the only one we need)
//
//  which gives:
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MsgType {
    BindingRequest = 0x0001,
    BindingResponse = 0x0101,
    BindingResponseError = 0x0111,
}

impl TryFrom<u16> for MsgType {
    type Error = u16;
    fn try_from(v: u16) -> StdResult<Self, Self::Error> {
        match v {
            x if x == MsgType::BindingRequest as u16 => Ok(MsgType::BindingRequest),
            x if x == MsgType::BindingResponse as u16 => Ok(MsgType::BindingResponse),
            x if x == MsgType::BindingResponseError as u16 => Ok(MsgType::BindingResponseError),
            _ => Err(v),
        }
    }
}

impl StunHeader {
    pub const LEN: usize = 20;

    const TYPE: Range<usize> = 0..2; // Big-endian u16.
    const LENGTH: Range<usize> = 2..4; // Big-endian u16.
    const COOKIE: Range<usize> = 4..8; // Big-endian u32.
    const TXID: Range<usize> = 8..20;

    fn set_message_type(&mut self, t: MsgType) -> &mut StunHeader {
        self.0[Self::TYPE].copy_from_slice(&(t as u16).to_be_bytes());
        self
    }

    fn message_type(&self) -> Result<MsgType, u16> {
        let m = u16::from_be_bytes(
            self.0[Self::TYPE]
                .try_into()
                .expect("cannot fail: msg type must be 2 bytes"),
        );

        m.try_into()
    }

    fn set_length(&mut self, l: u16) -> &mut StunHeader {
        self.0[Self::LENGTH].copy_from_slice(&l.to_be_bytes());
        self
    }

    fn length(&self) -> u16 {
        u16::from_be_bytes(
            self.0[Self::LENGTH]
                .try_into()
                .expect("cannot fail: length type must be 2 bytes"),
        )
    }

    fn set_magic_cookie(&mut self) -> &mut StunHeader {
        // The cookie is sent in network byte order.
        self.0[Self::COOKIE].copy_from_slice(&MAGIC_COOKIE.to_ne_bytes());
        self
    }

    fn set_transaction_id(&mut self) -> &mut StunHeader {
        let mut rnd = SystemRng;
        let mut id = [0u8; 12];
        rnd.fill_bytes(&mut id);
        self.0[Self::TXID].copy_from_slice(&id);
        self
    }

    fn transaction_id(&self) -> &[u8; 12] {
        self.0[Self::TXID]
            .first_chunk::<12>()
            .expect("cannot fail: transaction id must be 12 bytes")
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
pub fn send_stun_binding_request(
    log: &Logger,
    socket: &net::UdpSocket,
    server: (&str, u16),
) -> Result<Ipv4Addr> {
    let server: SocketAddr = server
        .to_socket_addrs()?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| anyhow!("no IPv4 address for stun.l.google.com"))?;

    let mut b = StunHeader::new_binding_request();

    info!(log, "Sending stun header to {server}:\n   {:02X?}", b.0);
    if let Err(e) = socket.send_to(&b.0, server) {
        warn!(log, "could not send: {e}");
        return Err(e).context("failed to send STUN binding response");
    }

    let mut buf = vec![0; 1500];
    let mut datagram;

    for tries in 0..10 {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) if addr == server.into() && n > StunHeader::LEN => {
                b = StunHeader(*buf.first_chunk::<20>().unwrap());
                datagram = &buf[0..n];
                break;
            }
            Err(e) => {
                warn!(log, "receive failed: {e}");
            }
            _ => {
                warn!(log, "received from other source");
            }
        }
    }

    Ok(Ipv4Addr::from_str("1.1.1.1").unwrap())
}

mod test {
    use crate::stun::*;

    #[test]
    fn test_header() {
        let mut s = StunHeader::new();
        s.set_message_type(MsgType::BindingRequest).set_length(10);

        assert_eq!(s.length(), 10);
        assert_eq!(s.message_type().ok(), Some(MsgType::BindingRequest));

        let s = StunHeader::new();
        assert_eq!(s.length(), 0);
        assert_eq!(s.message_type().ok(), None);
    }
}
