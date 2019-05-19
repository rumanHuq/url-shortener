use std::{env, net};

use hyper::{
  rt::{run, Future as Promise},
  service::service_fn,
  Body, Error as HyperError, Method, Request, Response, Server, StatusCode,
};

use log::{error, info};
use pretty_env_logger;

use futures::{future as promiseFn, Stream};

type Resolve = Response<Body>;
type Reject = HyperError;
type BoxedPromise = Box<Promise<Item = Resolve, Error = Reject> + Send>;

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
