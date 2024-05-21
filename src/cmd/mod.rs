use crate::{Connection, Frame, Parse};

mod echo;
use echo::Echo;

mod ping;
use ping::Ping;

mod unknown;
use unknown::Unknown;

#[derive(Debug)]
pub enum Command {
    Echo(Echo),
    Ping(Ping),
    Unknown(Unknown),
}

impl Command {
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match &command_name[..] {
            "echo" => Command::Echo(Echo::parse_frames(&mut parse)?),
            "ping" => Command::Ping(Ping::parse_frames(&mut parse)?),
            _ => {
                return Ok(Command::Unknown(Unknown::new(command_name)));
            }
        };

        parse.finish()?;

        Ok(command)
    }

    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        use Command::*;

        match self {
            Echo(cmd) => cmd.apply(dst).await,
            Ping(cmd) => cmd.apply(dst).await,
            Unknown(cmd) => cmd.apply(dst).await,
        }
    }
}
