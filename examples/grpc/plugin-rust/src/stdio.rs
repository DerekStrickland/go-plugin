use crate::proto::google::protobuf::Empty;
use crate::proto::plugin::grpc_stdio_server::GrpcStdio;
use crate::proto::plugin::stdio_data::Channel;
use crate::proto::plugin::StdioData;
use std::pin::Pin;
use std::time::Duration;

use futures_util::Stream;
use gag::BufferRedirect;
use std::io::Read;
// use tonic::codegen::{Context, Poll};
use std::thread::sleep;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// type StdioResult<T> = Result<Response<T>, Status>;
// type StdioStream = Pin<Box<dyn Stream<Item = Result<StdioData, Status>> + Send>>;

#[derive(Debug)]
pub struct StdioServer {}

impl core::default::Default for StdioServer {
    fn default() -> Self {
        StdioServer {}
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
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Self::StreamStdioStream>(4);

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
        let sleep_interval = Duration::from_milliseconds("10");

        tokio::spawn(async move {
            loop {
                // Read the stdout buffer
                let size = out_redirect.read(&mut out_data).unwrap();
                // Send the stdout bytes
                if size > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = out_data[..size].to_vec();
                    out_data.clear();
                    if let Err(e) = tx
                        .send(Self::StreamStdioStream::new(Box::new(response.clone())))
                        .await
                    {
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
                if size > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = err_data[..size].to_vec();
                    err_data.clear();
                    if let Err(e) = tx
                        .send(Self::StreamStdioStream::new(Box::new(response.clone())))
                        .await
                    {
                        status = Status::cancelled(e.to_string());
                    }
                }

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }

                sleep(sleep_interval)
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

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}
