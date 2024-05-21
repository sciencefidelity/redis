use crate::{Connection, Frame, Parse, ParseError};
use bytes::Bytes;

#[derive(Debug, Default)]
pub struct Echo {
    msg: Option<Bytes>,
}

impl Echo {
    pub fn new(msg: Option<Bytes>) -> Self {
        Echo { msg }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        match parse.next_bytes() {
            Ok(msg) => Ok(Self::new(Some(msg))),
            Err(ParseError::EndOfStream) => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let response = match self.msg {
            Some(msg) => Frame::Bulk(msg),
            None => Frame::Error(format!("ERR wrong number of arguments for 'echo' command")),
        };

        dst.write_frame(&response).await?;

        Ok(())
    }
}
