use async_stream::*;
use futures_core::Stream;

use tonic::{Request, Response, Status};

use crate::proto::google::protobuf::Empty;
use crate::proto::plugin::grpc_stdio_server::GrpcStdio;

use crate::proto::plugin::StdioData;
use std::pin::Pin;

use futures_util::AsyncBufReadExt;
use std::io;
use std::io::stdout;
// use crate::proto::plugin::stdio_data::Channel;
// use futures_util::StreamExt;
// use tokio::io::{AsyncReadExt, AsyncSeekExt};
// use tokio::signal::unix::signal;
// use tokio::signal::unix::SignalKind;

pub struct StdioServer {
    // shutdown_rx: oneshot::Receiver<()>,
// std_out: mpsc::UnboundedSender<()>,
// std_err: mpsc::UnboundedSender<()>,
}

impl core::default::Default for StdioServer {
    fn default() -> Self {
        StdioServer {
            // shutdown_rx,
            // std_out,
            // std_err,
        }
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
        // block on stdout/stderr, and terminate gracefully on interrupt.
        // TODO: This might require DropReceiver instead.
        // let data = try_stream! {
        //     let mut buf = String::new();
        //     _ = io::stdout().read_line(&mut buf)? => {
        //         yield StdioData{
        //             channel: Channel::Stdout as i32,
        //             data: buf.as_bytes()
        //         };
        //     },
        //     _ = io::stderr().read_line(&mut buf)? => {
        //         yield StdioData{
        //             channel: Channel::Stderr as i32,
        //             data: buf.as_bytes()
        //         };
        //     },
        //     _ = signal(SignalKind::interrupt())? => {
        //         yield StdioData {
        //             channel: Channel::Stdout as i32,
        //             data: String::from("").to_vec(),
        //         };
        //     }
        // };

        let mut buf = String::new();
        let mut stdout_stream = io::stdout()?;

        let data = tokio_select! {
            _ = stdout_stream.next() => {
                yield StdioData{
                    channel: Channel::Stdout as i32,
                    data: buf.as_bytes()
                };
            },
            _ = io::stderr().read_line(&mut buf)? => {
                yield StdioData{
                    channel: Channel::Stderr as i32,
                    data: buf.as_bytes()
                };
            },
            _ = signal(SignalKind::interrupt())? => {
                yield StdioData {
                    channel: Channel::Stdout as i32,
                    data: String::from("").to_vec(),
                };
            }
        };

        Ok(Response::new(Box::pin(data) as Self::StreamStdioStream))
    }
}

// let mut hangup = signal(SignalKind::hangup())?;
// let mut interrupt = signal(SignalKind::interrupt())?;
// let mut terminate = signal(SignalKind::terminate())?;
// let mut disconnect = signal(SignalKind)
//
// tokio::select! {
//     _ = hangup.recv() => {
//         let _ = log(String::from("SIGHUP received"));
//         tx.send(()).unwrap();
//     },
//     _ = interrupt.recv() => {
//         let _ = log(String::from("SIGINT received"));
//         tx.send(()).unwrap();
//     },
//     _ = terminate.recv() => {
//         let _ = log(String::from("SIGTERM received"));
//         tx.send(()).unwrap();
//     },
// }

// struct ClientDisconnect(oneshot::Sender<()>);
//
// impl Stream for ClientDisconnect {
//     type Item = Result<EchoResponse, Status>;
//
//     fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         // A stream that never resolves to anything....
//         Poll::Pending
//     }
// }

// pub struct DropReceiver<T> {
//     sender: oneshot::Sender<usize>,
//     inner: mpsc::Receiver<T>,
// }
//
// impl<T> Stream for DropReceiver<T> {
//     type Item = T;
//
//     fn poll_next(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut Context<'_>,
//     ) -> Poll<Option<Self::Item>> {
//         Pin::new(&mut self.inner).poll_recv(cx)
//     }
// }
//
// impl<T> Deref for DropReceiver<T> {
//     type Target = mpsc::Receiver<T>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }
//
// impl<T> Drop for DropReceiver<T> {
//     fn drop(&mut self) {
//         self.sender.send(1);
//     }
// }

// let (tx, rx) = mpsc::unbounded_channel();
// let mut signals = iterator::Signals::new(&[SIGINT])?;
//
// tokio::spawn(async move {
//     for sig in signals.forever() {
//         match sig {
//             SIGQUIT => tx.send(Ok(StdioData {
//                 channel: Channel::Stdout as i32,
//                 data: vec![],
//             })),
//             _ => continue,
//         }
//     }
// });
