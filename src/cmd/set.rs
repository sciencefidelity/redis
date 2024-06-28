use crate::{parse, Connection, Db, Frame, Parse};

use bytes::Bytes;
use tokio::time::Duration;

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
    expire: Option<Duration>,
}

impl Set {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        use parse::Error::EndOfStream;

        let key = parse.next_string()?;
        let value = parse.next_bytes()?;
        let mut expire = None;

        match parse.next_string() {
            Ok(s) if s.to_uppercase() == "EX" => {
                let secs = parse.next_int()?;
                expire = Some(Duration::from_secs(secs));
            }
            Ok(s) if s.to_uppercase() == "PX" => {
                let ms = parse.next_int()?;
                expire = Some(Duration::from_millis(ms));
            }
            Ok(_) => return Err("currently 'SET' only supports the expiration option".into()),
            Err(EndOfStream) => {}
            Err(err) => return Err(err.into()),
        }

        Ok(Self { key, value, expire })
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        db.set(self.key, self.value, self.expire);

        let response = Frame::Simple("OK".to_string());
        dst.write_frame(&response).await?;
        Ok(())
    }
}
