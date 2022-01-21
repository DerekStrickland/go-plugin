use std::pin::Pin;
use std::io::{Read};

use crate::proto::google::protobuf::Empty;
use crate::proto::plugin::grpc_stdio_server::GrpcStdio;
use crate::proto::plugin::stdio_data::Channel;
use crate::proto::plugin::StdioData;

use futures_util::{Stream};
use gag::BufferRedirect;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

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
        let (tx, rx) = mpsc::channel(4);

        // Flyweight initialized with empty data, and default channel to stdout
        let mut response: StdioData = StdioData {
            channel: Channel::Stdout as i32,
            data: b"intializing stdio stream".to_vec(),
        };

        if let Err(e) = tx.send(Ok(response.clone())).await {
            println!("unable to send stdio data: {}", e.to_string());
        }

        tokio::spawn(async move {
            // This is our inverse sentinel value we'll use to ensure that we can
            // detect an error condition has occurred.
            let sentinel = Uuid::new_v4().to_string();
            // This wraps our sentinel value to test if we've hit an error status.
            let mut status = Status::unknown(sentinel.clone());

            // TODO: Move to a global context so that it picks up other contexts.
            let mut buffer = Vec::new();
            let mut stdout_redirect = BufferRedirect::stdout().unwrap();
            let mut stderr_redirect = BufferRedirect::stderr().unwrap();
            let debug = true;

            loop {
                // TODO: Make this a configurable option.
                // This Debug Block is useful for proving the go-plugin GRPCClient
                // is receiving the channel data.
                if debug {
                    println!("should send stdout");
                    eprintln!("should send stderr")
                }

                stdout_redirect.read_to_end(&mut buffer).unwrap_or_else(|e| {
                    println!("error retrieving stdout: {}", e);
                    response.data = format!("error retrieving stdout: {}", e).as_bytes().to_vec();
                    return 0;
                });

                if buffer.len() > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = buffer[..].to_vec();
                    if let Err(e) = tx.send(Ok(response.clone())).await {
                        status = Status::cancelled(e.to_string());
                    }
                }

                buffer.clear();

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }

                stderr_redirect.read_to_end(&mut buffer).unwrap_or_else(|e| {
                    println!("error retrieving stderr: {}", e);
                    response.data = format!("error retrieving stderr: {}", e).as_bytes().to_vec();
                    return 0;
                });

                // send the stderr bytes
                if buffer.len() > 0 {
                    response.channel = Channel::Stderr as i32;
                    response.data = buffer[..].to_vec();
                    if let Err(e) = tx.send(Ok(response.clone())).await {
                        status = Status::cancelled(e.to_string());
                    }
                }

                buffer.clear();

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }
            }

            if !status.message().eq(&sentinel) {
                // TODO: Inspect client drop vs other errors.
                response.channel = Channel::Stderr as i32;
                response.data = status.message().as_bytes().to_vec();

                if let Err(e) = tx.send(Ok(response.clone())).await {
                    println!("unable to send status: {} - {}", status.message(), e.to_string());
                }
            }

            // TODO: Handle Client Drop
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}
