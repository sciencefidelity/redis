use crate::{Connection, Frame, Parse, ParseError};
use bytes::Bytes;

#[derive(Debug, Default)]
pub struct Echo {
    msg: Option<Bytes>,
}

impl Echo {
    pub fn new(msg: Option<Bytes>) -> Self {
        Self { msg }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        match parse.next_bytes() {
            Ok(msg) => Ok(Self::new(Some(msg))),
            Err(ParseError::EndOfStream) => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let response = self.msg.map_or_else(
            || Frame::Error("ERR wrong number of arguments for 'echo' command".to_string()),
            |msg| Frame::Bulk(msg),
        );

        dst.write_frame(&response).await?;

        Ok(())
    }
}
