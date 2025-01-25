use http_body_util::BodyExt;
use http_body_util::Empty;
use hyper::body::Buf;
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde::Deserialize;
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
    let is_json = match env::args().nth(2) {
        Some(is) => is,
        None => {
            println!("Usage: client <url> <is_json>");
            return Ok(());
        }
    }
    .parse::<bool>()
    .unwrap();

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
    let authority = url.authority().unwrap().clone();
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;

    println!("Response status: {}", res.status());

    if is_json {
        let body = res.collect().await?.aggregate();
        //from_reader(): The content of the I/O stream is deserialized directly from the stream without being buffered in memory
        let users: Vec<User> = serde_json::from_reader(body.reader())?;
        println!("users: {:?}", users);
    } else {
        //Stream the body, writing each frame to stdout as it arrives.
        while let Some(next) = res.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                io::stdout().write_all(&chunk).await?;
            }
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct User {
    #[allow(unused)]
    id: i32,
    #[allow(unused)]
    name: String,
}

//How to run
//1- cargo run --bin hs  //the hello server Listening to 127.0.0.1:3000
//2- cargo run --bin hc 127.0.0.1:3000 false  //the client and is_json_de
// To show json deserilaizer feture:
//cargo run --bin hc http://jsonplaceholder.typicode.com/users true //the client
