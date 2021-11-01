// use std::fs;
// use std::io::Write;
use std::net::SocketAddr;
// use std::pin::Pin;
// use std::task::{Context, Poll};

// use futures_core::Stream;
use futures_util::FutureExt;
use tokio::sync::oneshot;
// use tokio::signal::unix::{signal, SignalKind};
use tonic::transport::Server;
use tonic_health::server::HealthReporter;

use crate::proto::proto::kv_server::KvServer;
use kv::KV;
// use tokio::sync::mpsc;

mod kv;
mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // go-plugin requires this to be written to satisfy the handshake protocol.
    // https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md#4-output-handshake-information
    println!("1|1|tcp|127.0.0.1:5001|grpc");

    let (_, rx) = oneshot::channel::<()>();

    let addr: SocketAddr = "0.0.0.0:5001".parse().expect("SocketAddr parse");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<KvServer<KV>>().await;

    tokio::spawn(driver_service_status(health_reporter.clone()));

    let plugin_server = KV::default();

    //let join_handle = tokio::spawn(async move {
    Server::builder()
        .add_service(health_service)
        .add_service(KvServer::new(plugin_server))
        //.serve(addr)
        .serve_with_shutdown(addr, rx.map(drop)) //rx.recv().map(drop)
        .await?;
    // .unwrap();
    //});

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

    // join_handle.await.unwrap();

    Ok(())
}

// fn log(msg: String) -> tokio::io::Result<()> {
//     let file_name: String = "kv_log".to_owned();
//
//     let mut file = fs::OpenOptions::new()
//         .write(true)
//         .append(true)
//         .open(file_name)
//         .unwrap();
//
//     file.write_all(msg.as_bytes())
// }

// Implement a HealthReporter handler for tonic.
async fn driver_service_status(mut reporter: HealthReporter) {
    reporter.set_serving::<KvServer<KV>>().await;
}

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
