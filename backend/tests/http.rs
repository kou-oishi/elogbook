use actix_web::{http::StatusCode, test, web, App};
use elogbook::handlers::greet;

#[actix_web::test]
async fn greet_returns_backend_hint() {
    let app = test::init_service(App::new().route("/", web::get().to(greet))).await;
    let request = test::TestRequest::get().uri("/").to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}
