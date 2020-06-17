use hyper::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, HOST};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
// use hyper_tls::HttpsConnector;
use http::status::StatusCode;
use std::net::{Ipv4Addr, SocketAddr};
use structopt::StructOpt;

// use failure::ResultExt;
// use exitfailure::ExitFailure;

#[allow(dead_code)]
fn debug_request<T>(req: &Request<T>) -> () {
    println!("{}", req.method());
    println!("{}", req.uri());
    for (key, value) in req.headers().iter() {
        println!("{:?}: {:?}", key, value.clone());
    }
}

// Create new URI from path and query
fn new_uri(uri: &Uri) -> Result<Uri, http::uri::InvalidUri> {
    let req_path = &uri.path().to_string()[1..];
    let target_uri_str = match uri.query() {
        Some(query) => format!("http://{}?{}", req_path, query),
        None => format!("http://{}", req_path),
    };
    // ^ TODO handle full authority
    // ^ TODO handle scheme
    // Set target uri in original request
    target_uri_str.parse::<Uri>()
}

async fn run_request(
    req: Request<Body>,
    count: u8,
) -> Result<Response<Body>, hyper::Error> {
    if count == 0 {
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap())
    } else {
        match Client::new().request(req).await {
            Ok(mut res) => {
                let headers = res.headers_mut();
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
                return Ok(res);
            }
            Err(_) => {
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap());
            }
        }
    }
}

async fn proxy(mut req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    *req.uri_mut() = match new_uri(req.uri()) {
        Ok(u) => u,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap());
        }
    };
    // Add 'Host' header for HTTP/1.1
    let host = format!("{}", req.uri().host().unwrap());
    req.headers_mut()
        .insert(HOST, HeaderValue::from_str(&host).unwrap());

    run_request(req, 1).await
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(long = "host", default_value = "0.0.0.0")]
    host: Ipv4Addr,
    #[structopt(short = "p", long = "port", default_value = "4000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Cli::from_args();
    let addr = SocketAddr::from((args.host, args.port));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(proxy)) });
    let server = Server::bind(&addr).serve(make_svc);

    // And now add a graceful shutdown signal...
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}
