use crate::proto::google::protobuf::Empty;
use crate::proto::plugin::grpc_stdio_server::GrpcStdio;
use crate::proto::plugin::stdio_data::Channel;
use crate::proto::plugin::StdioData;
use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use gag::BufferRedirect;
use std::io::Read;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct StdioServer {
    // This is the channel we send data on.
    tx: tokio::sync::mpsc::UnboundedSender<_>,
    rx: tokio::sync::mpsc::UnboundedReceiver<_>,
}

impl core::default::Default for StdioServer {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        StdioServer { tx, rx }
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
        let mut status = Status::unknown(sentinel);
        let mut out_buf = BufferRedirect::stdout()?;
        let mut out_data = Vec::with_capacity(1024);
        let mut err_buf = BufferRedirect::stderr()?;
        let mut err_data = Vec::with_capacity(1024);

        tokio::spawn(async move {
            loop {
                // Read the stdout buffer
                out_buf.read(&mut out_data);
                // Send the stdout bytes
                if out_data.len() > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = out_data.clone();
                    out_data.clear();
                    match self.tx.send(Ok(response.clone())) {
                        Ok(_) => {}
                        Err(e) => status = Status::cancelled(e.to_string()),
                    }
                }

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }

                // Read the stderr buffer
                err_buf.read(&mut err_data);
                // send the stderr bytes
                if err_data.len() > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = err_data.clone();
                    err_data.clear();
                    match self.tx.send(Ok(response.clone())) {
                        Ok(_) => {}
                        Err(e) => status = Status::cancelled(e.to_string()),
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

        // Disconnect the stdio sinks.
        // tokio::io::stdout()
        //     .write_all(out_buf.into_inner().bytes())
        //     .await;
        // tokio::io::stderr()
        //     .write_all(err_buf.into_inner().bytes())
        //     .await;

        Ok(Response::new(
            Box::pin(UnboundedReceiverStream::new(self.rx)) as Self::StreamStdioStream,
        ))
    }
}

trait StdoutStream: Stream + StreamExt {}

// struct StdoutSink<W>(W);
//
// impl<W: AsyncWrite> Sink<I> for StdoutSink<W> {
//     // An error will be of this type:
//     type SinkError = std::io::Error;
//
//     // This is called to provide an item to the Sink. We might want to
//     // push it to a buffer here, but to keep things simple we just forward
//     // it on to the underlying `AsyncWrite` by calling `poll_write`. The item
//     // is returned if nothing can be done with it yet, which is why the return
//     // type is a little different here:
//     fn start_send(&mut self, item: u8) -> core::result::Result<AsyncSink<u8>, Self::SinkError> {
//         match self.0.poll_write(item: I)? {
//             Async::NotReady => Ok(AsyncSink::NotReady(item)),
//             Async::Ready(_) => Ok(AsyncSink::Ready),
//         }
//     }
//
//     // This is called after potentially multiple calls to `start_send`. Its goal is
//     // to flush the data out to ensure it's been fully written.
//     fn poll_complete(&mut self) -> core::result::Result<Async<()>, Self::SinkError> {
//         match self.0.poll_flush()? {
//             Async::Ready(_) => Ok(Async::Ready(())),
//             Async::NotReady => Ok(Async::NotReady),
//         }
//     }
// }

// let out_fd = FileDescriptor::dup(&stdout).unwrap();
// let err_fd = FileDescriptor::dup(&stderr).unwrap();
// let out_handle = stdout.lock();
// let err_handle = stderr.lock();
// let out_fd = FileDescriptor::dup(&out_handle).unwrap();
// let err_fd = FileDescriptor::dup(&err_handle).unwrap();

// let mut out_buf: Vec<u8> = Vec::new();
// let mut err_buf: Vec<u8> = Vec::new();
//loop {

//let err_bytes = err_tx.next().await;

//     match (
//         out_fd.buffer().read_to_end(&mut out_buf).await,
//         err_fd.buffer().read_to_end(&mut err_buf).await,
//     ) {
//         (Some(mut out_buf), None) => {
//             result.channel = Channel::Stdout as i32;
//             result.data = out_buf;
//         }
//         (None, Some(mut err_buf)) => {
//             result.channel = Channel::Stderr as i32;
//             result.data = err_buf;
//         }
//         (Some(mut out_buf), Some(mut err_buf)) => {
//             result.channel = Channel::Stdout as i32;
//             result.data = out_buf;
//
//             result.channel = Channel::Stderr as i32;
//             result.data = err_buf;
//         }
//         // (Err(err), None) => {
//         //     status = Status::unknown(err.to_string());
//         //     break;
//         // }
//         // (None, Err(err)) => {
//         //     status = Status::unknown(err.to_string());
//         //     break;
//         // }
//         _ => {
//             result.channel = Channel::Stdout as i32;
//             result.data = "".as_bytes().to_vec();
//             break;
//         }
//     }
//}

// let foo = stdout.poll_flush(cx).await;
// let out = stdout.with_subscriber();
// let mut out.write_all().await;
// let out_handle = tokio::spawn(async move { stdout.poll_flush() });
//
// let err_handle = tokio::spawn(async move {});
//
// out_handle.await;
// err_handle.await;

// drop(self.stdin.take());

// let mut data = match (
//     tokio::io::stdout().poll_flush(),
//     tokio::io::stderr().poll_flush(),
// ) {
//     (None, None) => data,
//     (Some(mut out), None) => {
//         let res = out.write_all(&mut stdout);
//         res.unwrap();
//         StdioData {
//             channel: Channel::Stdout as i32,
//             data: res,
//         }
//     }
//     (None, Some(mut err)) => {
//         let res = err.write_all(&mut stderr);
//         res.unwrap();
//         StdioData {
//             channel: Channel::Stderr as i32,
//             data: res,
//         }
//     }
//     // (Some(out), Some(err)) => {
//     //     let res = read2(out.inner, &mut stdout, err.inner, &mut stderr);
//     //     res.unwrap();
//     //     Err(err)
//     // },
//     _ => data, // return default instance
// };

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
