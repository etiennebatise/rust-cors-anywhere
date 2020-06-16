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

async fn proxy(mut req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // Create new URI from path and query
    let req_uri = req.uri();
    let req_path = &req_uri.path().to_string()[1..];
    let target_uri_str = match req_uri.query() {
        Some(query) => format!("http://{}?{}", req_path, query),
        None => format!("http://{}", req_path),
    };
    // ^ TODO handle full authority
    // ^ TODO handle scheme
    // Set target uri in original request
    let target_uri = match target_uri_str.parse::<Uri>() {
        Ok(u) => u,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap());
        }
    };
    *req.uri_mut() = target_uri;
    // Add 'Host' header for HTTP/1.1
    let host = format!("{}", req.uri().host().unwrap());
    req.headers_mut()
        .insert(HOST, HeaderValue::from_str(&host).unwrap());

    match Client::new().request(req).await {
        Ok(mut res) => {
            println!("{}", target_uri_str);
            let headers = res.headers_mut();
            headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
            return Ok(res)
        }
        Err(_) => {
            return Ok(Response::builder()
                      .status(StatusCode::INTERNAL_SERVER_ERROR)
                      .body(Body ::empty())
                      .unwrap());
        }
    }
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
