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
            // The only error status we expect at this time is a tx.send error
            // which indicates the client has disconnected.
            let mut status = Status::unknown(sentinel.clone());

            let mut out_redirect = BufferRedirect::stdout().unwrap();
            let mut out_data = Vec::with_capacity(1024);
            let mut err_redirect = BufferRedirect::stderr().unwrap();
            let mut err_data = Vec::with_capacity(1024);
            let debug = true;

            loop {
                // TODO: Make this a configurable option.
                // This Debug Block is useful for proving the go-plugin GRPCClient
                // is receiving the channel data.
                if debug {
                    // response.data = b"stdio listener loop".to_vec();
                    // if let Err(e) = tx.send(Ok(response.clone())).await {
                    //     println!("unable to send stdio data: {}", e.to_string());
                    // }

                    // if count.rem_euclid(5) == 0 {
                        println!("should send stdout foo");
                        eprintln!("should send stderr");
                    // }
                }

                out_redirect.read_to_end(&mut out_data).unwrap_or_else(|e| {
                    println!("error retrieving stdout: {}", e);
                    response.data = format!("error retrieving stdout: {}", e).as_bytes().to_vec();
                    return 0;
                });

                if out_data.len() > 0 {
                    response.channel = Channel::Stdout as i32;
                    response.data = out_data[..].to_vec();
                    out_data.clear();
                    if let Err(e) = tx.send(Ok(response.clone())).await {
                        status = Status::cancelled(e.to_string());
                    }
                }

                // Break on error.
                if !status.message().eq(&sentinel) {
                    break;
                }

                err_redirect.read_to_end(&mut err_data).unwrap_or_else(|e| {
                    println!("error retrieving stderr: {}", e);
                    response.data = format!("error retrieving stderr: {}", e).as_bytes().to_vec();
                    return 0;
                });

                // send the stderr bytes
                if err_data.len() > 0 {
                    response.channel = Channel::Stderr as i32;
                    response.data = err_data[..].to_vec();
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


// if !status.message().eq(&sentinel) {
// // // TODO: Inspect client drop vs other errors.
// // Ok(Response::new(Box::pin(status) as Self::StreamStdioStream))
// response.channel = Channel::Stderr as i32;
// response.data = status.message().as_bytes().to_vec();
// } else {
// // Can this ever fire? Need a shutdown broadcast receiver from main.
// response.channel = Channel::Stdout as i32;
// // TODO: Add a shutdown reason if detectable?
// response.data = "plugin shutdown".as_bytes().to_vec();
// }

// trait StdoutStream: Stream + StreamExt {}

// #[derive(Debug)]
// pub struct StdoutSink<W>(mpsc::Sender<Option<Vec<u8>>>);
//
// impl <W: AsyncWrite> AsyncWrite for StdoutSink<W> {
//     type Item = Vec<u8>;
//     type Error = Error;
//
//     fn start_send(&mut self, item: Vec<u8>) -> Result<AsyncSink<Vec<u8>>, Self::Error> {
//         match self.0.start_send(Some(item)) {
//             Err(_) => Err(ErrorKind::Other.into()),
//             Ok(AsyncSink::Ready) => Ok(AsyncSink::Ready),
//             Ok(AsyncSink::NotReady(o)) => Ok(AsyncSink::NotReady(o.unwrap())),
//         }
//     }
//
//     fn poll_complete(&mut self) -> Result<Async<()>, Self::Error> {
//         match self.0.poll_flush()? {
//             Err(_) => Err(ErrorKind::Other.into()),
//             Ok(x) => Ok(x),
//         }
//     }
//
//     fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
//         todo!()
//     }
//
//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }
//
//     fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }
// }

// let stdout_sink = drain()
// .with(|out_data: Vec<u8>| async move {
// response.channel = Channel::Stdout as i32;
// response.data = out_data.clone();
// if let Err(e) = tx.send(Ok(response.clone())).await {
// status = Status::cancelled(e.to_string());
// }
// });


// if let Err(e) = watch() {
//     status = Status::cancelled(e.to_string());
// }

// let out_size = std::io::stdout().as_filelike_view::<File>().read_vectored(&mut out_data).unwrap_or_else(|e| {
//     println!("{}", e);
//     return 0
// });
// // Read the stdout buffer
// // let size = out_redirect.read(&mut out_data).unwrap();
// // Send the stdout bytes
// if out_size > 0 {
//     response.channel = Channel::Stdout as i32;
//     response.data = out_data[out_size].to_vec();
//     out_data.clear();
//     if let Err(e) = tx.send(Ok(response.clone())).await {
//         status = Status::cancelled(e.to_string());
//     }
// }
//
// let out_take = std::io::stdout().take(1024).collect::<Vec<u8>>().await;
// if out_take.len() > 0 {
//     response.channel = Channel::Stdout as i32;
//     response.data = out_take;
//     // out_data.clear();
//     if let Err(e) = tx.send(Ok(response.clone())).await {
//         status = Status::cancelled(e.to_string());
//     }
// }

// let out_read = std::io::stdout().read_to_end(&mut out_data).unwrap_or_else(|e| {
//     println!("{}", e);
//     return 0
// });
// if out_read.unwrap() > 0 {
//     response.channel = Channel::Stdout as i32;
//     response.data = out_data.clone();
//     if let Err(e) = tx.send(Ok(response.clone())).await {
//         status = Status::cancelled(e.to_string());
//     }
// }
//
// std::io::stdout().read(&mut out_data).map(Ok).forward(stdout_sink).await;

// fn watch() -> notify::Result<()> {
//     // let path = get_executable_path();
//
//     let stdout_path = Path::new("/proc/self/fd/1");
//
//     // Create a channel to receive the events.
//     let (tx, rx) = std::sync::mpsc::channel::<DebouncedEvent>();
//
//     // Automatically select the best implementation for your platform.
//     // You can also access each implementation directly e.g. INotifyWatcher.
//     let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
//
//     // Add a path to be watched. All files and directories at that path and
//     // below will be monitored for changes.
//     watcher.watch(stdout_path, RecursiveMode::NonRecursive)?;
//
//     // This is a simple loop, but you may want to use more complex logic here,
//     // for example to handle I/O.
//     loop {
//         match rx.recv() {
//             Ok(event) => {
//                 println!("{:?}", event)
//             },
//             Err(e) => println!("watch error: {:?}", e),
//         }
//     }
// }

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
