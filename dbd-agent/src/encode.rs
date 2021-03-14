use crate::Frame;
use async_bincode::AsyncBincodeWriter;
use async_std::task;
use futures::SinkExt;
use tide::Error;
use tokio::io::{self, duplex, DuplexStream};
use tokio_stream::{Stream, StreamExt};

pub fn encode(
    mut stream: impl Stream<Item = io::Result<Frame>> + Unpin + Send + 'static,
) -> DuplexStream {
    let (reader, writer) = duplex(16384);

    task::spawn(async move {
        let mut sink = AsyncBincodeWriter::from(writer).for_async();

        while let Some(item) = stream.next().await {
            sink.send(item?).await?;
        }

        Ok::<_, Error>(())
    });

    reader
}
