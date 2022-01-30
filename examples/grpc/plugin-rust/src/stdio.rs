use std::pin::Pin;
use std::io::{Read};
use std::sync::{Arc, Mutex};

use crate::proto::google::protobuf::Empty;
use crate::proto::plugin::grpc_stdio_server::GrpcStdio;
use crate::proto::plugin::stdio_data::Channel;
use crate::proto::plugin::StdioData;

use futures_util::{Stream};
use gag::BufferRedirect;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct StdioServer {
    redirect: StdioRedirect,
}

impl StdioServer { // core::default::Default for
    pub fn new(redirect: StdioRedirect) -> Self {
        StdioServer {redirect}
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
            data: b"initializing stdio stream".to_vec(),
        };

        if let Err(e) = tx.send(Ok(response.clone())).await {
            println!("unable to send stdio data: {}", e.to_string());
        }

        // tokio::spawn(async move |response:StdioData|{
            // This is our inverse sentinel value we'll use to ensure that we can
            // detect an error condition has occurred.
            let sentinel = Uuid::new_v4().to_string();
            // This wraps our sentinel value to test if we've hit an error status.
            let mut status = Status::unknown(sentinel.clone());
            // TODO: Move to a global context so that it picks up other contexts.
            let debug = true;

            loop {
                // TODO: Make this a configurable option.
                // This Debug Block is useful for proving the go-plugin GRPCClient
                // is receiving the channel data.
                if debug {
                    // println!("should send stdout");
                    // eprintln!("should send stderr");
                }

                match self.redirect.read_stdout() {
                    Ok(buffer) => {
                        if !buffer.is_empty() {
                            response.channel = Channel::Stdout as i32;
                            response.data = buffer;
                            if let Err(e) = tx.send(Ok(response.clone())).await {
                                status = Status::cancelled(e.to_string());
                            }
                        }
                    },
                    Err(e) => {
                        status = Status::cancelled(e.to_string());
                    }
                }

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }

                match self.redirect.read_stderr() {
                    Ok(buffer) => {
                        if !buffer.is_empty() {
                            response.channel = Channel::Stderr as i32;
                            response.data = buffer;
                            if let Err(e) = tx.send(Ok(response.clone())).await {
                                status = Status::cancelled(e.to_string());
                            }
                        }
                    },
                    Err(e) => {
                        status = Status::cancelled(e.to_string());
                    }
                }

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
        //});

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

pub struct StdioRedirect {
    stdout: Arc<Mutex<BufferRedirect>>,
    stderr: Arc<Mutex<BufferRedirect>>,
}

impl StdioRedirect {
    pub fn new(stdout: BufferRedirect, stderr: BufferRedirect) -> Self {
        StdioRedirect{
            stdout: Arc::new(Mutex::new(stdout)),
            stderr: Arc::new(Mutex::new(stderr)),
        }
    }

    pub fn read_stdout(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer:Vec<u8> = Vec::new();
        let stdout = Arc::clone(&self.stdout);

        stdout.lock().unwrap().read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    pub fn read_stderr(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer:Vec<u8> = Vec::new();
        let stderr = Arc::clone(&self.stderr);

        stderr.lock().unwrap().read_to_end(&mut buffer)?;

        Ok(buffer)
    }
}
