use crate::{Connection, Db, Frame, Parse};

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Get> {
        let key = parse.next_string()?;

        Ok(Get { key })
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = if let Some(value) = db.get(&self.key) {
            Frame::Bulk(value)
        } else {
            Frame::Null
        };

        dst.write_frame(&response).await?;

        Ok(())
    }
}
