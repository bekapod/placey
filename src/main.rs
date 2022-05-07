
   
#![warn(clippy::all, clippy::pedantic)]
use warp::Filter;

fn index_route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
  warp::get().and(warp::path::end().map(|| "Hello, World!")).boxed()
}

fn generate_route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
  let image = warp::path("g");
  let rectangle = image.and(warp::path!(u16 / u16).map(|width, height| format!("rectangle: {}x{}", width, height)));
  let square = image.and(warp::path!(u16).and(warp::path::end()).map(|size| format!("square: {}x{}", size, size)));

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
  assert_eq!(res.body(), "rectangle: 150x300");
}

#[tokio::test]
async fn generate_square() {
  let req = warp::test::request().path("/g/150");
  let res = req.reply(&generate_route()).await;

  assert_eq!(res.status(), 200);
  assert_eq!(res.body(), "square: 150x150");
}