use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, Client, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::{HOST, ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue};

fn debug_request<T>(req: &Request<T>) -> () {
    println!("{}", req.method());
    println!("{}", req.uri());
    for (key, value) in req.headers().iter() {
        println!("{:?}: {:?}", key, value.clone());
    }
}

async fn hello_world(mut req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let uri : &str = &req.uri().to_string()[1..];
    // ^ TODO Dont trust user input ([1..] might fail)
    let target_url = format!("http://{}", uri);
    let cols : Vec<&str> = uri.splitn(2, '/').collect::<Vec<&str>>();
    let host = cols[0];
    // ^ TODO  Don't trust user input(uri might not contain '/')
    req.headers_mut().insert(HOST, HeaderValue::from_str(host).unwrap());
    *req.uri_mut() = target_url.parse::<Uri>().expect("failed to parse URL");
    // ^ TODO generate tokio crash when target_url is not a valid uri
    let mut res = Client::new().request(req).await?;
    let headers = res.headers_mut();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    Ok(res)
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(hello_world))
    });
    let server = Server::bind(&addr).serve(make_svc);

    // And now add a graceful shutdown signal...
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}
