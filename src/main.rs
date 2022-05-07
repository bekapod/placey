
   
#![warn(clippy::all, clippy::pedantic)]
use warp::{Filter, http::Response, http::Result};


mod image;

fn index_route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
  warp::get().and(warp::path::end().map(|| "Hello, World!")).boxed()
}

fn generate_route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
  fn get_image_response(w: u16, h: u16) -> Result<Response<Vec<u8>>> {
    match image::generate(w, h) {
      Ok((img, ext)) => Response::builder().header("content-type", format!("image/{}", ext)).body(img),
      Err((status, message)) => Response::builder().status(status).body(message.into())
    }
  }

  let image = warp::path("g");
  let rectangle = image.and(warp::path!(u16 / u16).map(get_image_response));
  let square = image.and(warp::path!(u16).and(warp::path::end()).map(|size| {
    get_image_response(size, size)
  }));

  warp::get().and(rectangle.or(square)).boxed()
}

#[tokio::main]
async fn main() {
  let end = index_route().with(warp::log("hello")).or(generate_route()).with(warp::log("generate"));

    warp::serve(end)
        // ipv6 + ipv6 any addr
        .run(([0, 0, 0, 0, 0, 0, 0, 0], 8080))
        .await;
}

#[tokio::test]
async fn landing() {
  let req = warp::test::request();
  let res = req.reply(&index_route()).await;

  assert_eq!(res.status(), 200);
  assert!(!res.body().is_empty());
}

#[tokio::test]
async fn generate_rectangle() {
  let req = warp::test::request().path("/g/150/300");
  let res = req.reply(&generate_route()).await;

  assert_eq!(res.status(), 200);
  assert!(!res.body().is_empty());
  assert_eq!(res.headers()["content-type"], "image/png");
}

#[tokio::test]
async fn generate_square() {
  let req = warp::test::request().path("/g/150");
  let res = req.reply(&generate_route()).await;

  assert_eq!(res.status(), 200);
  assert!(!res.body().is_empty());
  assert_eq!(res.headers()["content-type"], "image/png");
}