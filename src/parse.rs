use crate::Frame;

use bytes::Bytes;
use std::{fmt, str, string::ToString, vec};

pub struct Parse {
    parts: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub enum ParseError {
    EndOfStream,
    Other(crate::Error),
}

impl Parse {
    pub(crate) fn new(frame: Frame) -> Result<Self, ParseError> {
        let array = match frame {
            Frame::Array(array) => array,
            frame => return Err(format!("protocol error; expected array, got {frame:?}").into()),
        };

        Ok(Self {
            parts: array.into_iter(),
        })
    }

    fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or(ParseError::EndOfStream)
    }

    pub(crate) fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(data) => str::from_utf8(&data[..])
                .map(ToString::to_string)
                .map_err(|_| "protocol error; invalid string".into()),
            frame => {
                Err(format!("protocol error; expected simple or bulk frame, got {frame:?}").into())
            }
        }
    }

    pub(crate) fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
        match self.next()? {
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Frame::Bulk(data) => Ok(data),
            frame => {
                Err(format!("protocol error; expected simple or bulk frame, got {frame:?}",).into())
            }
        }
    }

    pub(crate) fn next_int(&mut self) -> Result<u64, ParseError> {
        match self.next()? {
            Frame::Bulk(data) => {
                let s = str::from_utf8(&data).expect("unaple to parse line");
                s.parse::<u64>()
                    .ok()
                    .ok_or_else(|| "protocol error; invalid number".into())
            }
            frame => Err(format!("protocol error; expected int frame but got {frame:?}").into()),
        }
    }

    pub(crate) fn finish(&mut self) -> Result<(), ParseError> {
        if self.parts.next().is_none() {
            Ok(())
        } else {
            Err("protocol error; expected end of frame, but there was more".into())
        }
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> Self {
        ParseError::Other(src.into())
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> Self {
        src.to_string().into()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            Self::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}
