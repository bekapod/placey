#![warn(clippy::all, clippy::pedantic)]
use placey::placeholder;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Response, http::Result, Filter};

fn index_route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let body = include_str!("ui/index.html");
    warp::get()
        .and(warp::path::end())
        .map(move || warp::reply::html(body))
        .boxed()
}

fn generate_route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    fn get_image_response(opts: placeholder::GenerateOptions) -> Result<Response<Vec<u8>>> {
        tracing::info!("route: generate");
        match placeholder::generate(opts) {
            Ok((img, ext)) => Response::builder()
                .header("content-type", format!("image/{}", ext))
                .body(img),
            Err((status, message)) => Response::builder().status(status).body(message.into()),
        }
    }

    let base = warp::path("g");

    let with_width = base.and(warp::path!(u16));
    let with_width_and_height = base.and(warp::path!(u16 / u16));
    let with_width_and_height_and_background_colour = base.and(warp::path!(u16 / u16 / String));
    let with_width_and_height_and_background_colour_and_foreground_colour =
        base.and(warp::path!(u16 / u16 / String / String));

    let square = with_width.map(|width| {
        get_image_response(placeholder::GenerateOptions {
            width,
            ..placeholder::GenerateOptions::default()
        })
    });
    let rectangle = with_width_and_height.map(|width, height| {
        get_image_response(placeholder::GenerateOptions {
            width,
            height: Some(height),
            ..placeholder::GenerateOptions::default()
        })
    });
    let rectangle_with_background_colour =
        with_width_and_height_and_background_colour.map(|width, height, background_colour| {
            get_image_response(placeholder::GenerateOptions {
                width,
                height: Some(height),
                background_colour: Some(background_colour),
                ..placeholder::GenerateOptions::default()
            })
        });
    let rectangle_with_background_colour_and_foreground_colour =
        with_width_and_height_and_background_colour_and_foreground_colour.map(
            |width, height, background_colour, foreground_colour| {
                get_image_response(placeholder::GenerateOptions {
                    width,
                    height: Some(height),
                    background_colour: Some(background_colour),
                    foreground_colour: Some(foreground_colour),
                })
            },
        );

    warp::get()
        .and(
            square.or(rectangle.or(rectangle_with_background_colour
                .or(rectangle_with_background_colour_and_foreground_colour))),
        )
        .boxed()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
    outer().await;
}

#[tracing::instrument]
async fn outer() {
    let end = warp::path("dist")
        .and(warp::fs::dir("./dist"))
        .or(index_route().or(generate_route()))
        .with(warp::log("request"))
        .with(warp::trace::request());

    warp::serve(end)
        // ipv6 + ipv6 any addr
        .run(([0, 0, 0, 0, 0, 0, 0, 0], 8080))
        .await;
}

#[tokio::test]
async fn landing() {
    let request = warp::test::request();
    let response = request.reply(&index_route()).await;

    assert_eq!(response.status(), 200);
    assert!(!response.body().is_empty());
}

#[tokio::test]
async fn generate_square() {
    let request = warp::test::request().path("/g/150");
    let response = request.reply(&generate_route()).await;

    assert_eq!(response.status(), 200);
    assert!(!response.body().is_empty());
    assert_eq!(response.headers()["content-type"], "image/png");
}

#[tokio::test]
async fn generate_rectangle() {
    let request = warp::test::request().path("/g/150/300");
    let response = request.reply(&generate_route()).await;

    assert_eq!(response.status(), 200);
    assert!(!response.body().is_empty());
    assert_eq!(response.headers()["content-type"], "image/png");
}

#[tokio::test]
async fn generate_rectangle_with_background_colour() {
    let request = warp::test::request().path("/g/150/300/fff000");
    let response = request.reply(&generate_route()).await;

    assert_eq!(response.status(), 200);
    assert!(!response.body().is_empty());
    assert_eq!(response.headers()["content-type"], "image/png");
}

#[tokio::test]
async fn generate_rectangle_with_background_colour_and_foreground_colour() {
    let request = warp::test::request().path("/g/150/300/fff000/ff1200");
    let response = request.reply(&generate_route()).await;

    assert_eq!(response.status(), 200);
    assert!(!response.body().is_empty());
    assert_eq!(response.headers()["content-type"], "image/png");
}
