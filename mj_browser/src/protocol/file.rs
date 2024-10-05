use std::path::PathBuf;

use tokio::{fs::File, io::AsyncReadExt};
use tracing::{event, instrument, Level};

pub struct MjFileProtocolHandler;

pub struct MjFileProtocolHandlerState {
    file: File,
}

impl MjFileProtocolHandler {
    #[instrument(skip(self))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        event!(Level::INFO, "Starting file protocol handler");
        let file = File::open(args).await?;
        Ok(MjFileProtocolHandlerState { file })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MjProtocolMessage::Read(reply) => {
                let mut buf = String::new();
                state.file.read_to_string(&mut buf).await;
                reply.send(Box::new(buf))?;
                myself.stop(None);
            }
            MjProtocolMessage::Write => todo!(),
        }
        Ok(())
    }
}