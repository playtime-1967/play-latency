use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening to {}", addr);

    loop {
        //Waits for a new incoming TCP connection on the listener socket
        //The server processes new connections sequentially and avoids race conditions or idle tasks.
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream); //adapter

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(hello_service))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

//A Service lets us define how our server will respond to incoming requests. It represents an async function that takes a Request and returns a Future.
//When the processing of this future is complete, it will resolve to a Response or an error.
async fn hello_service(
    request: Request<hyper::body::Incoming>,
    //std::convert::Infallible: A type used to indicate a situation where no error can occur.
) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("request {:?}", request);
    Ok(Response::new(Full::new(Bytes::from(
        "Hello, HTTP1 Server! \n",
    ))))
}
