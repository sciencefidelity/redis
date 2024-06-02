use crate::{Connection, Db, Frame, Parse};

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;

        Ok(Self { key })
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = db
            .get(&self.key)
            .map_or_else(|| Frame::Null, |value| Frame::Bulk(value));

        dst.write_frame(&response).await?;

        Ok(())
    }
}
