use std::future::Future;
use std::net::SocketAddr;

use futures_util::FutureExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_health::server::HealthReporter;

use crate::store::Store;

pub fn run(listener: TcpListener, shutdown: impl Future) {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<KvServer<Store>>().await;

    tokio::spawn(driver_service_status(health_reporter.clone()));

    let addr: SocketAddr = "0.0.0.0:5001".parse().expect("SocketAddr parse");
    let plugin_server = Store::default();

    let (_, rx) = unbounded_channel();

    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(health_service)
            .add_service(KvServer::new(plugin_server))
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), rx.map(drop))
            .await
            .unwrap();
    });

    server.await.expect("server shutdown")
}

// Implement a HealthReporter handler for tonic.
async fn driver_service_status(mut reporter: HealthReporter) {
    reporter.set_serving::<KvServer<Store>>().await;
}
