use anyhow::Result;
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::get,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::template::{self, Template};

#[derive(OpenApi)]
#[openapi(info(title = "ODRL renderer and validator"))]
pub struct ArunaApi;

pub fn router(templates: Arc<Vec<Template>>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(render_pdf))
        .routes(routes!(validate_odrl))
        .with_state(templates)
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Term {
    pub heading: String,
    pub text: String,
}

/// Render a data usage agreement as a PDF.
#[utoipa::path(
    post,
    path = "/validate",
    request_body = serde_json::Value, //odrl::model::policy::AgreementPolicy,
    responses(
        (status = 200, content_type = "text/xml", body = Vec<u8>),
    ),
)]
pub async fn validate_odrl(Json(request): Json<serde_json::Value>) -> impl IntoResponse {
    let result = crate::validate::validate_odrl(request);
    let mut headers = HeaderMap::new();

    match result {
        Ok(pdf) => {
            headers.insert(header::CONTENT_TYPE, "text/xml".parse().unwrap());
            (StatusCode::OK, headers, pdf)
        }
        Err(e) => {
            eprintln!("Failed to render PDF: {:?}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                e.to_string().as_bytes().to_vec(),
            )
        }
    }
}

/// Render a data usage agreement as a PDF.
#[utoipa::path(
    post,
    path = "/render",
    request_body = serde_json::Value, //odrl::model::policy::AgreementPolicy,
    responses(
        (status = 200, content_type = "application/pdf", body = Vec<u8>),

    ),
)]
pub async fn render_pdf(
    State(state): State<Arc<Vec<Template>>>,
    Json(request): Json<generic_odrl::policy::GenericPolicy>,
) -> impl IntoResponse {
    let result = template::render_pdf(request, state);
    //let result: std::result::Result<Vec<u8>, anyhow::Error> = Ok(vec![]);

    let mut headers = HeaderMap::new();

    match result {
        Ok(pdf) => {
            headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"contract.pdf\"".parse().unwrap(),
            );
            (StatusCode::OK, headers, pdf)
        }
        Err(e) => {
            eprintln!("Failed to render PDF: {:?}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                e.to_string().as_bytes().to_vec(),
            )
        }
    }
}

pub async fn run() -> Result<()> {
    let socket_address = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(socket_address).await.unwrap();
    let templates = Arc::new(template::load_templates().await?);
    let (router, api) = OpenApiRouter::with_openapi(ArunaApi::openapi())
        .nest("/api", router(templates))
        .split_for_parts();

    let swagger = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api);

    let app = router
        .route("/api", get(|| async { Redirect::permanent("/swagger-ui") }))
        .merge(swagger)
        .layer(
            TraceLayer::new_for_http()
                .on_response(())
                .on_body_chunk(())
                .on_eos(()),
        );
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
