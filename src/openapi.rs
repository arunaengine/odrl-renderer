use anyhow::Result;
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::get,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::template::{self, Template};

#[derive(OpenApi)]
#[openapi(info(
    title = "ODRL renderer and validator",
    license(name = "MIT", url = "https://opensource.org/license/mit/"),
    version = "0.1.0"
))]
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

/// Validate a ODRL Set | Offer | Agreement policy. Returns a JUnit XML report.
#[utoipa::path(
    post,
    path = "/validate",
    request_body(
        description = "ODRL policy to validate",
        content(
            (serde_json::Value = "application/json+ld", example = json!(
  {
  "@context": "https://www.w3.org/ns/odrl.jsonld",
  "@type": "Set",
  "uid": "",
  "assignee": "https://orcid.org/0000-0002-1825-0097",
  "assigner": {
    "uid": "https://orcid.org/0000-0002-1825-0096"
  },
  "target": {
    "uid": "https://doi.org/10.2154/123456"
  },
  "permission": [],
  "prohibition": [],
  "obligation": [
    {
      "@type": "Rule",
      "action": "cc:Notice",
      "constraint": []
    },
    {
      "@type": "Rule",
      "action": "o-dd:attribution",
      "constraint": []
    }
  ]
})),
        ),
    ),
    responses(
        (status = 200, content_type = "text/xml", body = Vec<u8>, example = r#"<testsuites>
    <testsuite id="0" name="odrl validation" package="testsuite/odrl validation" tests="1" errors="0" failures="0" hostname="localhost" timestamp="2024-12-12T14:28:07.005251217Z" time="0.000000001">
      <testcase name="valid odrl" time="0.000000001"/>
    </testsuite>
  </testsuites>"#),
    ),
    tag = "odrl"
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
    request_body(
        description = "ODRL policy to validate",
        content(
            (serde_json::Value = "application/json+ld", example = json!(
  {
  "@context": "https://www.w3.org/ns/odrl.jsonld",
  "@type": "Set",
  "uid": "",
  "assignee": "https://orcid.org/0000-0002-1825-0097",
  "assigner": {
    "uid": "https://orcid.org/0000-0002-1825-0096"
  },
  "target": {
    "uid": "https://doi.org/10.2154/123456"
  },
  "permission": [],
  "prohibition": [],
  "obligation": [
    {
      "@type": "Rule",
      "action": "cc:Notice",
      "constraint": []
    },
    {
      "@type": "Rule",
      "action": "o-dd:attribution",
      "constraint": []
    }
  ]
})),
        ),
    ),
    responses(
        (status = 200, content_type = "application/pdf", description = "A rendered pdf with odrl.jsonld as attachment", body = Vec<u8>),
    ),
    tag = "odrl"
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
        )
        .layer(CorsLayer::very_permissive());
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
