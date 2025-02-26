use std::future::Future;
use std::task::{Context, Poll};

use actix_utils::future::{ok, Ready};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::{http, web::Path, App, Error, HttpResponse, Responder};
use actix_web_codegen::{connect, delete, get, head, options, patch, post, put, route, trace};
use futures_core::future::LocalBoxFuture;

// Make sure that we can name function as 'config'
#[get("/config")]
async fn config() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/test")]
async fn test_handler() -> impl Responder {
    HttpResponse::Ok()
}

#[put("/test")]
async fn put_test() -> impl Responder {
    HttpResponse::Created()
}

#[patch("/test")]
async fn patch_test() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/test")]
async fn post_test() -> impl Responder {
    HttpResponse::NoContent()
}

#[head("/test")]
async fn head_test() -> impl Responder {
    HttpResponse::Ok()
}

#[connect("/test")]
async fn connect_test() -> impl Responder {
    HttpResponse::Ok()
}

#[options("/test")]
async fn options_test() -> impl Responder {
    HttpResponse::Ok()
}

#[trace("/test")]
async fn trace_test() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/test")]
fn auto_async() -> impl Future<Output = Result<HttpResponse, actix_web::Error>> {
    ok(HttpResponse::Ok().finish())
}

#[get("/test")]
fn auto_sync() -> impl Future<Output = Result<HttpResponse, actix_web::Error>> {
    ok(HttpResponse::Ok().finish())
}

#[put("/test/{param}")]
async fn put_param_test(_: Path<String>) -> impl Responder {
    HttpResponse::Created()
}

#[delete("/test/{param}")]
async fn delete_param_test(_: Path<String>) -> impl Responder {
    HttpResponse::NoContent()
}

#[get("/test/{param}")]
async fn get_param_test(_: Path<String>) -> impl Responder {
    HttpResponse::Ok()
}

#[route("/multi", method = "GET", method = "POST", method = "HEAD")]
async fn route_test() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/custom_resource_name", name = "custom")]
async fn custom_resource_name_test<'a>(req: actix_web::HttpRequest) -> impl Responder {
    assert!(req.url_for_static("custom").is_ok());
    assert!(req.url_for_static("custom_resource_name_test").is_err());
    HttpResponse::Ok()
}

pub struct ChangeStatusCode;

impl<S, B> Transform<S, ServiceRequest> for ChangeStatusCode
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ChangeStatusCodeMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ChangeStatusCodeMiddleware { service })
    }
}

pub struct ChangeStatusCodeMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ChangeStatusCodeMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();
            let header_name = HeaderName::from_lowercase(b"custom-header").unwrap();
            let header_value = HeaderValue::from_str("hello").unwrap();
            headers.insert(header_name, header_value);
            Ok(res)
        })
    }
}

#[get("/test/wrap", wrap = "ChangeStatusCode")]
async fn get_wrap(_: Path<String>) -> impl Responder {
    // panic!("actually never gets called because path failed to extract");
    HttpResponse::Ok()
}

#[actix_rt::test]
async fn test_params() {
    let srv = actix_test::start(|| {
        App::new()
            .service(get_param_test)
            .service(put_param_test)
            .service(delete_param_test)
    });

    let request = srv.request(http::Method::GET, srv.url("/test/it"));
    let response = request.send().await.unwrap();
    assert_eq!(response.status(), http::StatusCode::OK);

    let request = srv.request(http::Method::PUT, srv.url("/test/it"));
    let response = request.send().await.unwrap();
    assert_eq!(response.status(), http::StatusCode::CREATED);

    let request = srv.request(http::Method::DELETE, srv.url("/test/it"));
    let response = request.send().await.unwrap();
    assert_eq!(response.status(), http::StatusCode::NO_CONTENT);
}

#[actix_rt::test]
async fn test_body() {
    let srv = actix_test::start(|| {
        App::new()
            .service(post_test)
            .service(put_test)
            .service(head_test)
            .service(connect_test)
            .service(options_test)
            .service(trace_test)
            .service(patch_test)
            .service(test_handler)
            .service(route_test)
            .service(custom_resource_name_test)
    });
    let request = srv.request(http::Method::GET, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::HEAD, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::CONNECT, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::OPTIONS, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::TRACE, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::PATCH, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::PUT, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());
    assert_eq!(response.status(), http::StatusCode::CREATED);

    let request = srv.request(http::Method::POST, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());
    assert_eq!(response.status(), http::StatusCode::NO_CONTENT);

    let request = srv.request(http::Method::GET, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::GET, srv.url("/multi"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::POST, srv.url("/multi"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::HEAD, srv.url("/multi"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());

    let request = srv.request(http::Method::PATCH, srv.url("/multi"));
    let response = request.send().await.unwrap();
    assert!(!response.status().is_success());

    let request = srv.request(http::Method::GET, srv.url("/custom_resource_name"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn test_auto_async() {
    let srv = actix_test::start(|| App::new().service(auto_async));

    let request = srv.request(http::Method::GET, srv.url("/test"));
    let response = request.send().await.unwrap();
    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn test_wrap() {
    let srv = actix_test::start(|| App::new().service(get_wrap));

    let request = srv.request(http::Method::GET, srv.url("/test/wrap"));
    let mut response = request.send().await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(response.headers().contains_key("custom-header"));
    let body = response.body().await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("wrong number of parameters"));
}
