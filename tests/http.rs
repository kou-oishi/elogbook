use actix_web::{http::StatusCode, test, web, App};
use elogbook::{api::HEALTH_PATH, server::handlers::health};

#[actix_web::test]
async fn health_returns_server_hint() {
    let app = test::init_service(App::new().route(HEALTH_PATH, web::get().to(health))).await;
    let request = test::TestRequest::get().uri(HEALTH_PATH).to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}
