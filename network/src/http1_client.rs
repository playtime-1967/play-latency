use http_body_util::BodyExt;
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::TokioIo;
use std::env;
use tokio::io::{self, AsyncWriteExt as _};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return Ok(());
        }
    };
    let url = url.parse::<hyper::Uri>().unwrap();

    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);

    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await?;

    // Use an adapter to access something implementing `tokio::io` traits as if they implement `hyper::rt` IO traits.
    let io = TokioIo::new(stream);

    // Create the Hyper client
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    //What Happens If conn.await Is Not Spawned:
    //The connection will block the task until it finishes (e.g., due to an error or when the connection is explicitly closed).
    //By spawning, it runs concurrently in its own lightweight task
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    // Create an HTTP request with an empty body and a HOST header
    let req = Request::builder()
        .header(
            hyper::header::HOST,
            url.authority().unwrap().clone().as_str(),
        )
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;

    println!("Response status: {}", res.status());

    //Stream the body, writing each frame to stdout as it arrives.
    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            io::stdout().write_all(&chunk).await?;
        }
    }
    Ok(())
}

//How to run
//1- cargo run --bin hts1  //the http1 server 127.0.0.1:3000
//2- cargo run --bin htc1 127.0.0.1:3000  //the client
