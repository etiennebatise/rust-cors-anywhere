use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, Client, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::{ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue};

async fn hello_world(mut req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let uri_str = format!("{}", req.uri());
    let url_str = format!("http://{}", &uri_str[1..]);
    let url = url_str.parse::<Uri>().expect("failed to parse URL");
    // ^ Generate thread panic when uri is empty
    *req.uri_mut() = url;
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
