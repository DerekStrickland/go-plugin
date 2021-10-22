use tokio::net::TcpListener;

mod proto;
mod server;
mod store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // go-plugin requires this to be written to satisfy the handshake protocol.
    // https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md#4-output-handshake-information
    println!("1|1|tcp|127.0.0.1:5001|grpc");

    let listener: TcpListener = TcpListener::bind(addr).await.expect("bind");

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
