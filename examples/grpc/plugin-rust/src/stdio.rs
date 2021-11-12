use crate::proto::google::protobuf::Empty;
use crate::proto::plugin::grpc_stdio_server::GrpcStdio;
use crate::proto::plugin::stdio_data::Channel;
use crate::proto::plugin::StdioData;
use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use gag::BufferRedirect;
use std::io::Read;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct StdioServer {
    // This is the channel we send data on.
// tx: tokio::sync::mpsc::UnboundedSender<_>,
// rx: tokio::sync::mpsc::UnboundedReceiver<_>,
}

impl core::default::Default for StdioServer {
    fn default() -> Self {
        //let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        StdioServer {} //tx, rx }
    }
}

#[tonic::async_trait]
impl GrpcStdio for StdioServer {
    type StreamStdioStream =
        Pin<Box<dyn Stream<Item = Result<StdioData, Status>> + Send + Sync + 'static>>;

    async fn stream_stdio(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::StreamStdioStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        // Flyweight initialized with empty data, and default channel to stdout
        let mut response: StdioData = StdioData {
            channel: Channel::Stdout as i32,
            data: "".as_bytes().to_vec(),
        };

        // This is our inverse sentinel value we'll use to ensure that we can
        // detect an error condition has occurred.
        let sentinel = Uuid::new_v4().to_string();
        // This wraps our sentinel value to test if we've hit an error status.
        // The only error status we expect at this time is a tx.send error
        // which indicates the client has disconnected.
        let mut status = Status::unknown(sentinel.clone());
        let mut out_redirect = BufferRedirect::stdout()?;
        let mut out_data = Vec::with_capacity(1024);
        let mut err_redirect = BufferRedirect::stderr()?;
        let mut err_data = Vec::with_capacity(1024);

        tokio::spawn(async move {
            loop {
                // Read the stdout buffer
                let size = out_redirect.read(&mut out_data).unwrap();
                // Send the stdout bytes
                if out_data.len() > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = out_data[..size].to_vec();
                    out_data.clear();
                    if let Err(e) = tx.send(Ok(response.clone())).await {
                        status = Status::cancelled(e.to_string());
                    }
                }

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }

                // Read the stderr buffer
                let size = err_redirect.read(&mut err_data).unwrap();
                // send the stderr bytes
                if err_data.len() > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = err_data[..size].to_vec();
                    err_data.clear();
                    if let Err(e) = tx.send(Ok(response.clone())).await {
                        status = Status::cancelled(e.to_string());
                    }
                }

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }
            }

            if !status.message().eq(&sentinel) {
                // // TODO: Inspect client drop vs other errors.
                // Ok(Response::new(Box::pin(status) as Self::StreamStdioStream))
                response.channel = Channel::Stderr as i32;
                response.data = status.message().as_bytes().to_vec();
            } else {
                // Can this ever fire? Need a shutdown broadcast receiver from main.
                response.channel = Channel::Stdout as i32;
                // TODO: Add a shutdown reason if detectable?
                response.data = "plugin shutdown".as_bytes().to_vec();
            }
        });

        // TODO: Seems like we need a clean exit, but not sure how yet.
        // Anecdotal evidence that this will cause problems. To test, from a
        // terminal do a watch ps aux | grep kv-
        // Spectacular weirdness that requires you to reset your terminal.
        // Disconnect the stdio sinks.
        // tokio::io::stdout()
        //     .write_all(out_buf.into_inner().bytes())
        //     .await;
        // tokio::io::stderr()
        //     .write_all(err_buf.into_inner().bytes())
        //     .await;

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}
