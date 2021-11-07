use crate::proto::google::protobuf::Empty;
use crate::proto::plugin::grpc_stdio_server::GrpcStdio;
use crate::proto::plugin::stdio_data::Channel;
use crate::proto::plugin::StdioData;
use std::pin::Pin;

use futures_util::stream::{Stream, TryStreamExt};
use tokio_util::io::ReaderStream;
use tonic::{Request, Response, Status};

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
    // type StreamStdioStream = BoxStream<'static, Result<StdioData, Status>>;
    type StreamStdioStream =
        Pin<Box<dyn Stream<Item = Result<StdioData, Status>> + Send + Sync + 'static>>;

    async fn stream_stdio(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::StreamStdioStream>, Status> {
        let stream = ReaderStream::new(tokio::io::stdin())
            .map_ok(|buf| StdioData {
                channel: Channel::Stdout as i32,
                data: buf.to_vec(),
            })
            .map_err(|err| Status::unknown(err.to_string()));

        let stream = Box::pin(stream);

        Ok(Response::new(stream))
    }
}

// struct ClientDisconnect(tokio::sync::mpsc::UnboundedSender<()>);
//
// impl Stream for ClientDisconnect {
//     type Item = Result<StdioData, Status>;
//
//     fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         // A stream that never resolves to anything....
//         Poll::Pending
//     }
// }
//
// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
// let mut stdout = io::stdout();
// let mut stderr = io::stderr();
// // Endlessly stream stdout/stderr until interrupt received
// loop {
// tokio::select! {
// buf = stdout.read_to_end().await => {
// tx.send(StdioData{
// channel: Channel::Stdout as i32,
// data: buf.as_bytes()
// });
// continue
// },
// buf = stderr.read_to_end().await => {
// tx.send(StdioData{
// channel: Channel::Stderr as i32,
// data: buf.as_bytes()
// });
// continue
// },
// _ = shutdown_rx.recv() => break,
// _ = (Err(e), _) | (_, Err(e)) => {
// tx.Send(Err(e.into()));
// continue
// }
// None => break,
// }
// }
//
// Ok(Response::new(
// Box::pin(ClientDisconnect(tx)) as Self::StreamStdioStream
// ))
// //Ok(Response::new(Box::pin(data) as Self::StreamStdioStream))
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

// let mut buf = String::new();
// let mut stdout_stream = io::stdout();
// stdout_stream.
//
// let stdout = io::stdout().poll_flush_unpin();
//
// let data = tokio_select! {
//     Some(buf) = stdout_stream.poll_flush_unpin() => {
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

// let mut stdout = FramedWrite::new(io::stdout(), BytesCodec::new());
// let mut stderr = FramedWrite::new(io::stderr(), BytesCodec::new());
// // let mut sink = FramedWrite::new(w, BytesCodec::new());
// // filter map Result<BytesMut, Error> stream into just a Bytes stream to match stdout Sink
// // on the event of an Error, log the error and end the stream
// let mut out_stream = FramedRead::new(io::stdout(), BytesCodec::new())
// .filter_map(|i| match i {
// //BytesMut into Bytes
// Ok(i) => future::ready(Some(i.freeze())),
// Err(e) => {
// println!("failed to read from stdout; error={}", e);
// future::ready(None)
// }
// })
// .map(Ok);
//
// let mut err_stream = FramedRead::new(io::stderr(), BytesCodec::new())
// .filter_map(|i| match i {
// //BytesMut into Bytes
// Ok(i) => future::ready(Some(i.freeze())),
// Err(e) => {
// println!("failed to read from stderr; error={}", e);
// future::ready(None)
// }
// })
// .map(Ok);
