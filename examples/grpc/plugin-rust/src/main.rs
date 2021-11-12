// use std::fs;
// use std::io::Write;
use std::net::SocketAddr;
use futures_util::FutureExt;
use tonic::transport::Server;
use tonic_health::server::HealthReporter;

use crate::proto::plugin::grpc_stdio_server::GrpcStdioServer;
use crate::proto::proto::kv_server::KvServer;
use kv::KV;
use stdio::StdioServer;
use tokio::signal::unix::{signal, SignalKind};

mod kv;
mod proto;
mod stdio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // go-plugin requires this to be written to satisfy the handshake protocol.
    // https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md#4-output-handshake-information
    println!("1|1|tcp|127.0.0.1:5001|grpc");

    let mut interrupt = signal(SignalKind::interrupt())?;
    //
    // let rx = DropReceiver {
    //     sender: oneshot_tx,
    //     inner: rx,
    // };

    let addr: SocketAddr = "0.0.0.0:5001".parse().expect("SocketAddr parse");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<KvServer<KV>>().await;

    tokio::spawn(driver_service_status(health_reporter.clone()));

    let kv_server = KV::default();
    let stdio_server = StdioServer::default();

    Server::builder()
        .add_service(health_service)
        .add_service(KvServer::new(kv_server))
        .add_service(GrpcStdioServer::new(stdio_server))
        .serve_with_shutdown(addr, interrupt.recv().map(drop))
        .await?;

    Ok(())
}

// Implement a HealthReporter handler for tonic.
async fn driver_service_status(mut reporter: HealthReporter) {
    reporter.set_serving::<KvServer<KV>>().await;
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
