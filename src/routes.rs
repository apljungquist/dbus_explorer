use axum::{routing::get, Router};

use crate::handlers::{all_services_page, landing_page, object_page, service_page};

pub fn create_routes() -> Router {
    Router::new()
        .route("/local/dbus_explorer/app", get(landing_page))
        .route("/local/dbus_explorer/app/", get(landing_page))
        .route("/local/dbus_explorer/app/all", get(all_services_page))
        .route(
            "/local/dbus_explorer/app/service/{service_name}",
            get(service_page),
        )
        .route(
            "/local/dbus_explorer/app/service/{service_name}/{*object_path}",
            get(object_page),
        )
}
