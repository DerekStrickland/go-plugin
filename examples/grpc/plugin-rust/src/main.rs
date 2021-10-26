// use futures_util::FutureExt;
use std::net::SocketAddr; //, TcpListener};

// use tokio::sync::oneshot;
use tonic::transport::Server;
use tonic_health::server::HealthReporter;

use crate::proto::proto::kv_server::KvServer;
use kv::KV;

mod kv;
mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // go-plugin requires this to be written to satisfy the handshake protocol.
    // https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md#4-output-handshake-information
    println!("1|1|tcp|127.0.0.1:5001|grpc");

    let addr: SocketAddr = "0.0.0.0:5001".parse().expect("SocketAddr parse");
    //let listener: TcpListener = TcpListener::bind(addr).await.expect("bind");

    //let (_, shutdown_rx) = oneshot::channel::<()>();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<KvServer<KV>>().await;

    tokio::spawn(driver_service_status(health_reporter.clone()));

    let plugin_server = KV::default();

    //let server = tokio::spawn(async move {
    Server::builder()
        .add_service(health_service)
        .add_service(KvServer::new(plugin_server))
        .serve(addr)
        //.serve_with_incoming_shutdown(listener, shutdown_rx.map(drop))
        .await
        .unwrap();
    //});

    // server.await.expect("server shutdown");

    Ok(())
}

// Implement a HealthReporter handler for tonic.
async fn driver_service_status(mut reporter: HealthReporter) {
    reporter.set_serving::<KvServer<KV>>().await;
}
