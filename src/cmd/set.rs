use crate::{Connection, Db, Frame, Parse, ParseError};

use bytes::Bytes;

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
}

impl Set {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        use ParseError::EndOfStream;

        let key = parse.next_string()?;
        let value = parse.next_bytes()?;

        match parse.next_string() {
            Ok(_) => return Err("currently 'SET' supports no options".into()),
            Err(EndOfStream) => {}
            Err(err) => return Err(err.into()),
        }

        Ok(Set { key, value })
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        db.set(self.key, self.value);

        let response = Frame::Simple("OK".to_string());
        dst.write_frame(&response).await?;
        Ok(())
    }
}
