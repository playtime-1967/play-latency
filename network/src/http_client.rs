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

    let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;

    let io = TokioIo::new(stream); //tokio adapter

    //create a Hyper client.
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    //run connections concurrently.
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    //create an HTTP request with an empty body and a HOST header
    let authority = url.authority().unwrap().clone();
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;

    println!("Response status: {}", res.status());

    //a JSON deserialization sample.
    if is_json {
        let body = res.collect().await?.aggregate();
        //content of the I/O stream is deserialized directly from the stream without being buffered in memory.
        let users: Vec<User> = serde_json::from_reader(body.reader())?;
        println!("users: {:#?}", users);
    } else {
        //stream the body, writing each frame as it arrives.
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
