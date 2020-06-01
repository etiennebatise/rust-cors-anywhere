use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, Client, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::{ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue};

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let uri_str = format!("{}", _req.uri());
    let url_str = format!("http://{}", &uri_str[1..]);
    // FIXME : Fails silently if url url_str cannot be parsed
    let url = url_str.parse::<Uri>().expect("failed to parse URL");
    let mut res = Client::new().get(url).await?;
    let headers = res.headers_mut();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    Ok(res)
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(hello_world))
    });
    let server = Server::bind(&addr).serve(make_svc);
    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
