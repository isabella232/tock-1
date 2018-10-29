#![allow(dead_code, unused_variables, unused_imports)]
#![no_std]

extern crate core as std;
#[macro_use]
pub extern crate cauterize;
use self::cauterize::{Cauterize, Decoder, Encoder, Error, Primitive, Range, Vector};
use std::mem;

pub static SPEC_NAME: &'static str = "msg";
pub const SPEC_FINGERPRINT: [u8; 20] = [
    0x15, 0x4d, 0xac, 0x61, 0x58, 0x0e, 0xf7, 0xe8, 0x81, 0x21, 0xca, 0x67, 0xb2, 0x44, 0x5f, 0x75,
    0xae, 0x9e, 0x2b, 0x9c,
];
pub const SPEC_MIN_SIZE: usize = 1;
pub const SPEC_MAX_SIZE: usize = 215;

impl_range!(TxPwr, u8, u8, 14, 49);

impl Cauterize for TxPwr {
    const FINGERPRINT: [u8; 20] = [
        0xeb, 0x64, 0x73, 0x89, 0xb2, 0x4d, 0x8c, 0xf2, 0x8f, 0x84, 0xfa, 0xa3, 0x36, 0x42, 0xf6,
        0x0b, 0x0a, 0x66, 0x50, 0x0e,
    ];
    const SIZE_MIN: usize = 1;
    const SIZE_MAX: usize = 1;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        let tag = (self.0 - 14) as u8;
        Ok(tag.encode(ctx)?)
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let tag = u8::decode(ctx)? as u8;
        TxPwr::new(tag + 14)
    }
}

impl_vector!(Payload, u8, 200);

impl Cauterize for Payload {
    const FINGERPRINT: [u8; 20] = [
        0xe8, 0x45, 0xee, 0x89, 0xdc, 0xb2, 0xe9, 0xd5, 0xfa, 0xcb, 0x51, 0x12, 0xda, 0xae, 0xc8,
        0xa3, 0x1e, 0xb5, 0x0b, 0xd4,
    ];
    const SIZE_MIN: usize = 1;
    const SIZE_MAX: usize = 201;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        if self.len > 200 {
            return Err(Error::ElementCount);
        }
        (self.len as u8).encode(ctx)?;
        for i in 0..self.len {
            self.elems[i].encode(ctx)?
        }
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let len = u8::decode(ctx)? as usize;
        if len > 200 {
            return Err(Error::ElementCount);
        }
        let mut v = Payload::new();
        for _ in 0..len {
            v.push(u8::decode(ctx)?);
        }
        Ok(v)
    }
}

impl_array!(Addr, u8, 10);

impl Cauterize for Addr {
    const FINGERPRINT: [u8; 20] = [
        0xea, 0x0d, 0xda, 0x38, 0x21, 0x29, 0x99, 0xbb, 0x35, 0x50, 0x72, 0x20, 0xd4, 0xbc, 0xc0,
        0xd8, 0xaa, 0x19, 0xef, 0xef,
    ];
    const SIZE_MIN: usize = 10;
    const SIZE_MAX: usize = 10;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        let ref elems = self.0;
        for elem in elems.iter() {
            elem.encode(ctx)?;
        }
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let mut arr: [u8; 10] = unsafe { mem::uninitialized() };
        for i in 0..10 {
            arr[i] = u8::decode(ctx)?;
        }
        Ok(Addr(arr))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ping {
    pub id: u8,        // 1
    pub address: Addr, // 2
    pub seq: u8,       // 3
    pub data: Payload, // 4
}

impl Cauterize for Ping {
    const FINGERPRINT: [u8; 20] = [
        0x66, 0xe4, 0x43, 0x50, 0x24, 0x4a, 0xb8, 0x02, 0xc0, 0x3a, 0xac, 0xbc, 0xeb, 0x20, 0x7a,
        0x68, 0xbe, 0xaa, 0x2b, 0x2d,
    ];
    const SIZE_MIN: usize = 13;
    const SIZE_MAX: usize = 213;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        self.id.encode(ctx)?;
        self.address.encode(ctx)?;
        self.seq.encode(ctx)?;
        self.data.encode(ctx)?;
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let rec = Ping {
            id: u8::decode(ctx)?,
            address: Addr::decode(ctx)?,
            seq: u8::decode(ctx)?,
            data: Payload::decode(ctx)?,
        };
        Ok(rec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pong {
    pub id: u8,        // 1
    pub address: Addr, // 2
    pub seq: u8,       // 3
}

impl Cauterize for Pong {
    const FINGERPRINT: [u8; 20] = [
        0xc7, 0x0b, 0x71, 0xf4, 0xd5, 0x60, 0x3a, 0x3f, 0x80, 0x19, 0xdc, 0xab, 0x3b, 0x27, 0x8d,
        0x05, 0xf1, 0x76, 0xd8, 0x01,
    ];
    const SIZE_MIN: usize = 12;
    const SIZE_MAX: usize = 12;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        self.id.encode(ctx)?;
        self.address.encode(ctx)?;
        self.seq.encode(ctx)?;
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let rec = Pong {
            id: u8::decode(ctx)?,
            address: Addr::decode(ctx)?,
            seq: u8::decode(ctx)?,
        };
        Ok(rec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pingpong {
    Ping(Ping), // 1
    Pong(Pong), // 2
}

impl Cauterize for Pingpong {
    const FINGERPRINT: [u8; 20] = [
        0xbb, 0xdf, 0x24, 0x48, 0x68, 0x67, 0x6c, 0x3a, 0x8a, 0x26, 0x77, 0xc6, 0xef, 0x00, 0x3c,
        0x94, 0x93, 0x94, 0xaa, 0x47,
    ];
    const SIZE_MIN: usize = 13;
    const SIZE_MAX: usize = 214;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        match self {
            &Pingpong::Ping(ref val) => {
                let tag: u8 = 1;
                tag.encode(ctx)?;
                val.encode(ctx)?;
            }
            &Pingpong::Pong(ref val) => {
                let tag: u8 = 2;
                tag.encode(ctx)?;
                val.encode(ctx)?;
            }
        };
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let tag = u8::decode(ctx)?;
        match tag {
            1 => Ok(Pingpong::Ping(Ping::decode(ctx)?)),
            2 => Ok(Pingpong::Pong(Pong::decode(ctx)?)),
            _ => Err(Error::InvalidTag),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Pingpong(Pingpong), // 1
}

impl Cauterize for Frame {
    const FINGERPRINT: [u8; 20] = [
        0xdd, 0x34, 0xd5, 0x59, 0xfa, 0x72, 0xfd, 0x2a, 0xd5, 0xab, 0x47, 0x56, 0xad, 0xae, 0x61,
        0xf0, 0x1b, 0xf5, 0x80, 0xa6,
    ];
    const SIZE_MIN: usize = 14;
    const SIZE_MAX: usize = 215;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        match self {
            &Frame::Pingpong(ref val) => {
                let tag: u8 = 1;
                tag.encode(ctx)?;
                val.encode(ctx)?;
            }
        };
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let tag = u8::decode(ctx)?;
        match tag {
            1 => Ok(Frame::Pingpong(Pingpong::decode(ctx)?)),
            _ => Err(Error::InvalidTag),
        }
    }
}
