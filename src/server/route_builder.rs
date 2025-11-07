use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

async fn proxy_to_nextjs(mut req: Request) -> Response {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let proxy_url =
        std::env::var("SERVER_PROXY_URL").unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));

    let proxy_uri = match proxy_url.parse::<hyper::Uri>() {
        Ok(uri) => uri,
        Err(e) => {
            tracing::error!("Invalid proxy URL {}: {}", proxy_url, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid proxy configuration",
            )
                .into_response();
        }
    };

    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(path);

    let new_uri = format!("{}{}", proxy_url, path_query);
    match new_uri.parse() {
        Ok(uri) => *req.uri_mut() = uri,
        Err(e) => {
            tracing::error!("Failed to parse URI {}: {}", new_uri, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI").into_response();
        }
    }

    if let Some(host) = proxy_uri.host() {
        let host_value = if let Some(port) = proxy_uri.port_u16() {
            format!("{}:{}", host, port)
        } else {
            host.to_string()
        };
        if let Ok(header_value) = host_value.parse() {
            req.headers_mut().insert(hyper::header::HOST, header_value);
        }
    }

    let client = Client::builder(TokioExecutor::new()).build_http();

    match client.request(req).await {
        Ok(response) => response.into_response(),
        Err(e) => {
            tracing::error!("Proxy error: {}", e);
            (StatusCode::BAD_GATEWAY, "Server not available").into_response()
        }
    }
}

pub async fn register_routes() -> Router {
    let api_routes = crate::api::routes();

    Router::new()
        .nest("/api", api_routes)
        .fallback(proxy_to_nextjs)
}
