use std::{env, net, str, sync::{Arc,RwLock}, collections::HashMap};

use hyper::{
  rt::{run, Future as Promise},
  service::service_fn,
  Body, Error as HyperError, Method, Request, Response, Server, StatusCode,
};

use log::{error, info};
use pretty_env_logger;

use futures::{future as promiseFn, Stream};
mod shortener;
use shortener::get_shortend_url;
use lazy_static::lazy_static;

type Resolve = Response<Body>;
type Reject = HyperError;
type BoxedPromise = Box<Promise<Item = Resolve, Error = Reject> + Send>;


type UrlDb=Arc<RwLock<HashMap<String, String>>>;

lazy_static! {
  static ref SHORT_URLS: UrlDb = Arc::new(RwLock::new(HashMap::new()));
}

fn app(req: Request<Body>) -> BoxedPromise {
  let mut response = Response::new(Body::empty());

  match (req.method(), req.uri().path()) {
    (&Method::GET, "/") => {
      *response.body_mut() = Body::from("Try POSTing data to /echo");
    }
    (&Method::POST, "/echo") => {
      *response.body_mut() = req.into_body();
    }
    (&Method::POST, "/echo/uppercase") => {
      let body = req.into_body().map(|chunk| {
        chunk
          .iter()
          .map(|byte| byte.to_ascii_uppercase())
          .collect::<Vec<u8>>()
      });
      *response.body_mut() = Body::wrap_stream(body)
    }
    (&Method::POST, "/echo/reverse") => {
      let reversed = req.into_body().concat2().map(move |chunk| {
        let body = chunk.iter().rev().cloned().collect::<Vec<u8>>();

        *response.body_mut() = Body::from(body);
        response
      });
      return Box::new(reversed);
    }
    (&Method::POST, "/url-shorten") =>{
     let response = req.into_body().concat2().map(move |chunk|{
       let cloned_chunk:Vec<u8> = chunk.iter().cloned().collect();
       let url_to_shorten = str::from_utf8(&cloned_chunk).unwrap();
       let shortened_url = get_shortend_url(url_to_shorten);
       SHORT_URLS.write().unwrap().insert(shortened_url, url_to_shorten.to_string());
       let body = &*SHORT_URLS.read().unwrap();
       if let Some(url) = body.keys().nth(0) {
        Response::new(Body::from(format!("http://127.0.0.1:3000/{}", url)))
       } else {
        Response::new(Body::from(format!("{:#?}", body)))
       }
     }); 
     return Box::new(response);
    }
    _ => {
      *response.status_mut() = StatusCode::NOT_FOUND;
    }
  };

  Box::new(promiseFn::ok(response))
}

fn main() {
  env::set_var("RUST_LOG", "url_shortener=info");
  pretty_env_logger::init();

  let address: net::SocketAddr = ([127, 0, 0, 1], 3000).into();
  let server_promise = Server::bind(&address)
    .serve(|| service_fn(app))
    .map_err(|e| error!("server error: {}", e));

  info!("URL shortener listening on: {}", address);
  run(server_promise);
}
