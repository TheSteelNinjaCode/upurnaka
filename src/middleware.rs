use axum::{extract::Request, http::HeaderValue, response::Response};
use rahti::request_pipeline::{Next, RequestPipeline};

async fn private_responses(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
}

pub fn pipeline() -> RequestPipeline {
    RequestPipeline::new().with(private_responses)
}
