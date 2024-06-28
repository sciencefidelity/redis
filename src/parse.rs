use crate::Frame;

use bytes::Bytes;
use std::{fmt, str, string::ToString, vec};

pub struct Parse {
    parts: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub enum Error {
    EndOfStream,
    Other(crate::Error),
}

impl Parse {
    pub(crate) fn new(frame: Frame) -> Result<Self, Error> {
        let array = match frame {
            Frame::Array(array) => array,
            frame => return Err(format!("protocol error; expected array, got {frame:?}").into()),
        };

        Ok(Self {
            parts: array.into_iter(),
        })
    }

    fn next(&mut self) -> Result<Frame, Error> {
        self.parts.next().ok_or(Error::EndOfStream)
    }

    pub(crate) fn next_string(&mut self) -> Result<String, Error> {
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

    pub(crate) fn next_bytes(&mut self) -> Result<Bytes, Error> {
        match self.next()? {
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Frame::Bulk(data) => Ok(data),
            frame => {
                Err(format!("protocol error; expected simple or bulk frame, got {frame:?}",).into())
            }
        }
    }

    pub(crate) fn next_int(&mut self) -> Result<u64, Error> {
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

    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        if self.parts.next().is_none() {
            Ok(())
        } else {
            Err("protocol error; expected end of frame, but there was more".into())
        }
    }
}

impl From<String> for Error {
    fn from(src: String) -> Self {
        Self::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Self {
        src.to_string().into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            Self::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
