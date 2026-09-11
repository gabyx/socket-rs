use std::{
    net::{self, Ipv4Addr},
    random::{Rng, SystemRng},
    str::FromStr,
};

use anyhow::Result;

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
struct StunHeader {
    msg_type: u16,
    msg_length: u16,
    cookie: u32,
    transaction_id: [u8; 12],
}

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
const MSG_TYPE_BINDING_REQUEST: u16 = 0x0001;
const MSG_TYPE_BINDING_SUCCESS_RESPONSE: u16 = 0x0101;
const MSG_TYPE_BINDING_ERROR_RESPONSE: u16 = 0x0111;

impl StunHeader {
    fn new_binding_request() -> StunHeader {
        let mut rnd = SystemRng;
        let mut id = [0u8; 12];
        rnd.fill_bytes(&mut id);

        StunHeader {
            msg_type: MSG_TYPE_BINDING_REQUEST,
            msg_length: 0,
            cookie: MAGIC_COOKIE,
            transaction_id: id,
        }
    }
}

// impl<'a> Into<&'a [u8]> for StunHeader {
//     fn into(self) -> &'a [u8] {}
// }

// Sends a STUN message to a public stun server to discover our IP
// address.
pub fn send_stun(socket: &net::UdpSocket) -> Result<Ipv4Addr> {
    // socket.send_to()
    Ok(Ipv4Addr::from_str("1.1.1.1").unwrap())
}
