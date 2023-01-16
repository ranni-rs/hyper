#![deny(warnings)]
#![warn(rust_2018_idioms)]

use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use serde::Deserialize;
use tokio::net::TcpStream;

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "http://localhost:8000/signed".parse().unwrap();
    let users = fetch_json(url).await?;
    // print users
    println!("users: {:#?}", users);

    // print the sum of ids
    let sum = users.iter().fold(0, |acc, user| acc + user.id);
    println!("sum of ids: {}", sum);
    Ok(())
}

async fn fetch_json(url: hyper::Uri) -> Result<Vec<User>> {
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(addr).await?;

    let (mut sender, conn) = hyper::client::conn::http2::handshake(stream).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    // Fetch the url...
    let req = Request::builder()
        .method("POST")
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body("".to_string())?;
    // let (res, write_at) = sender.send_request(req).await?;
    let res = sender.send_request(req).await?;

    // asynchronously aggregate the chunks of the body
    let _body = res.collect().await?.aggregate();

    // try to parse as json with serde_json
    // let users = serde_json::from_reader(body.reader())?;

    // let now = chrono::Local::now().timestamp_nanos();
    // println!(
    //     "Cost: {} nano sec.\n about: {:.6} ms",
    //     now - write_at,
    //     (now - write_at) as f64 / 1000.0 / 1000.0
    // );
    Ok(vec![])
}

#[derive(Deserialize, Debug)]
struct User {
    id: i32,
    #[allow(unused)]
    name: String,
}
