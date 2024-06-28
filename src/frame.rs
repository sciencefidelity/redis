use bytes::{Buf, Bytes};
use std::io::Cursor;
use std::num::TryFromIntError;
use std::str;
use std::string::FromUtf8Error;

#[derive(Clone, Debug)]
pub enum Frame {
    Array(Vec<Frame>),
    Bulk(Bytes),
    Error(String),
    Null,
    Simple(String),
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}

impl Frame {
    /// # Errors
    ///
    /// Will return `Err` if fails to get the next byte.
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    skip(src, 4)
                } else {
                    let len: usize = get_decimal(src)?.try_into()?;
                    skip(src, len + 2)
                }
            }
            b'*' => {
                let len = get_decimal(src)?;
                for _ in 0..len {
                    Self::check(src)?;
                }
                Ok(())
            }
            actual => Err(format!("protocol error; invalid frame type byte `{actual}").into()),
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if fails to get the next byte.
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        match get_u8(src)? {
            b'+' => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)?;
                Ok(Self::Simple(string))
            }
            b'-' => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)?;
                Ok(Self::Error(string))
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;
                    if line != b"-1" {
                        return Err("protocol error; invalid frame format".into());
                    }

                    Ok(Self::Null)
                } else {
                    let len = get_decimal(src)?.try_into()?;
                    let n = len + 2;

                    if src.remaining() < n {
                        return Err(Error::Incomplete);
                    }

                    let data = Bytes::copy_from_slice(&src.chunk()[..len]);

                    skip(src, n)?;
                    Ok(Self::Bulk(data))
                }
            }
            b'*' => {
                let len = get_decimal(src)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Self::parse(src)?);
                }

                Ok(Self::Array(out))
            }
            _ => unimplemented!(),
        }
    }
}

fn peek_u8(src: &Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    let line = get_line(src)?;

    let s = str::from_utf8(line).expect("unaple to parse line");
    s.parse::<u64>()
        .ok()
        .ok_or_else(|| "protocol error; invalid frame format".into())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = usize::try_from(src.position())?;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
}

impl From<String> for Error {
    // TODO: fix this unconditional recursion
    #[allow(clippy::unconditional_recursion)]
    fn from(src: String) -> Self {
        src.into()
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Self {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Self {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Self {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Incomplete => "stream ended early".fmt(fmt),
            Self::Other(err) => err.fmt(fmt),
        }
    }
}
