#![no_std]

use core::convert::TryInto;
use core::str::{from_utf8, Utf8Error};

use half::f16;

pub use crate::token::{Encoding, Token};

mod token;

pub struct Parser<'a> {
    src: &'a [u8],
}

impl<'a> Parser<'a> {
    pub fn from_slice(src: &[u8]) -> Parser {
        Parser { src }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (&byte, rest) = self.src.split_first()?;
        self.src = rest;
        Some(self.parse_token(byte))
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Copy, Clone, Debug)]
pub struct Error;

impl<'a> Parser<'a> {
    fn take(&mut self, mid: u64) -> Result<&'a [u8]> {
        if mid > self.src.len() as u64 {
            return Err(Error);
        }
        let (left, right) = self.src.split_at(mid as usize);
        self.src = right;
        Ok(left)
    }

    fn take_text(&mut self, mid: u64) -> Result<&'a str> {
        from_utf8(self.take(mid)?).map_err(Into::into)
    }

    fn take_u8(&mut self) -> Result<u8> {
        Ok(u8::from_be_bytes(self.take(1)?.try_into().unwrap()))
    }

    fn take_u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn take_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn take_u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn parse_token(&mut self, byte: u8) -> Result<Token<'a>> {
        use crate::token::Encoding::*;
        use crate::token::Token::*;
        Ok(match byte {
            // Unsigned integers
            0x00..=0x17 => Unsigned(byte.into(), SameByte),
            0x18 => Unsigned(self.take_u8()?.into(), OneByte),
            0x19 => Unsigned(self.take_u16()?.into(), TwoBytes),
            0x1a => Unsigned(self.take_u32()?.into(), FourBytes),
            0x1b => Unsigned(self.take_u64()?, EightBytes),
            0x1c..=0x1f => return Err(Error),
            // Negative integers
            0x20..=0x37 => Negative((byte - 0x20).into(), SameByte),
            0x38 => Negative(self.take_u8()?.into(), OneByte),
            0x39 => Negative(self.take_u16()?.into(), TwoBytes),
            0x3a => Negative(self.take_u32()?.into(), FourBytes),
            0x3b => Negative(self.take_u64()?, EightBytes),
            0x3c..=0x3f => return Err(Error),
            // Byte string
            0x40..=0x57 => Bytes(self.take((byte - 0x40).into())?, SameByte),
            0x58 => Bytes(self.take_u8().and_then(|v| self.take(v.into()))?, OneByte),
            0x59 => Bytes(self.take_u16().and_then(|v| self.take(v.into()))?, TwoBytes),
            0x5a => Bytes(
                self.take_u32().and_then(|v| self.take(v.into()))?,
                FourBytes,
            ),
            0x5b => Bytes(self.take_u64().and_then(|v| self.take(v))?, EightBytes),
            0x5c..=0x5e => return Err(Error),
            0x5f => StartBytes,
            // UTF-8 string
            0x60..=0x77 => Text(from_utf8(self.take((byte - 0x60).into())?)?, SameByte),
            0x78 => Text(
                self.take_u8().and_then(|v| self.take_text(v.into()))?,
                OneByte,
            ),
            0x79 => Text(
                self.take_u16().and_then(|v| self.take_text(v.into()))?,
                TwoBytes,
            ),
            0x7a => Text(
                self.take_u32().and_then(|v| self.take_text(v.into()))?,
                FourBytes,
            ),
            0x7b => Text(self.take_u64().and_then(|v| self.take_text(v))?, EightBytes),
            0x7c..=0x7e => return Err(Error),
            0x7f => StartText,
            // Array
            0x80..=0x97 => StartArray(Some(((byte - 0x80).into(), SameByte))),
            0x98 => StartArray(Some((self.take_u8()?.into(), OneByte))),
            0x99 => StartArray(Some((self.take_u16()?.into(), TwoBytes))),
            0x9a => StartArray(Some((self.take_u32()?.into(), FourBytes))),
            0x9b => StartArray(Some((self.take_u64()?, EightBytes))),
            0x9c..=0x9e => return Err(Error),
            0x9f => StartArray(None),
            // Map
            0xa0..=0xb7 => StartMap(Some(((byte - 0xa0).into(), SameByte))),
            0xb8 => StartMap(Some((self.take_u8()?.into(), OneByte))),
            0xb9 => StartMap(Some((self.take_u16()?.into(), TwoBytes))),
            0xba => StartMap(Some((self.take_u32()?.into(), FourBytes))),
            0xbb => StartMap(Some((self.take_u64()?, EightBytes))),
            0xbc..=0xbe => return Err(Error),
            0xbf => StartMap(None),
            // Tag
            0xc0..=0xd7 => Tag((byte - 0xc0).into()),
            0xd8 => Tag(self.take_u8()?.into()),
            0xd9 => Tag(self.take_u16()?.into()),
            0xda => Tag(self.take_u32()?.into()),
            0xdb => Tag(self.take_u64()?),
            0xdc..=0xdf => return Err(Error),
            // Other
            0xe0..=0xf7 => SimpleValue(byte - 0xe0),
            0xf8 => SimpleValue(self.take_u8()?),
            0xf9 => Half(f16::from_bits(self.take_u16()?)),
            0xfa => Single(f32::from_bits(self.take_u32()?)),
            0xfb => Double(f64::from_bits(self.take_u64()?)),
            0xfc..=0xfe => return Err(Error),
            0xff => Stop,
        })
    }
}

impl From<Utf8Error> for Error {
    fn from(_error: Utf8Error) -> Self {
        Self
    }
}
