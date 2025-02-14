use http_body_util::Empty;
use http_body_util::Full;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Body;
use hyper::body::Bytes;
use hyper::body::Frame;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5948));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening to {}", addr);

    loop {
        //process new connections sequentially and avoids race conditions or idle tasks.
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream); //adapter

        tokio::task::spawn(async move {
            if let Err(err) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, service_fn(echo_service))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn echo_service(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    //a simple Routing Table
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full(
            "Welcome, We have the following endpoints: \n POST echo, POST echo/uppercase, POST echo/reverse",
        ))),
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body().boxed())),
        (&Method::POST, "/echo/uppercase") => {
            let frame_stream = req.into_body().map_frame(|frame| {
                let frame = if let Ok(data) = frame.into_data() {
                    data.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Bytes>()
                } else {
                    Bytes::new()
                };

                Frame::data(frame)
            });

            Ok(Response::new(frame_stream.boxed()))
        }
        (&Method::POST, "/echo/reverse") => {
            //protect the server from massive bodies.
            let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if upper > 1024 * 64 {
                let mut resp = Response::new(full("Body is too big!"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            //await the whole body to be collected
            let whole_body = req.collect().await?.to_bytes();

            //reverse the body.
            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();

            Ok(Response::new(full(reversed_body)))
        }
        //return 404 for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

//BoxBody<D,E>: a boxed, dynamically-dispatched implementation of the Body trait in Hyper.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    //Empty: a body that is always empty, means Empty will never produce an error when used.
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
