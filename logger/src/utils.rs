use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tracing::*;

pub async fn request_with_span<
    Fut: std::future::Future<Output = Result<T, E>>,
    T,
    E: std::fmt::Display,
>(
    request: &str,
    fut: Fut,
) -> Result<T, E> {
    let req_id: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();
    let req_id = req_id.as_str();
    let span = tracing::error_span!("req", req_id);
    let _enter = span.enter();

    debug!("Start request [{}]", request);

    fut.await
        .map_err(|err| {
            error!("Request error: {}", err);
            err
        })
        .map(|res| {
            debug!("Request completed successfully");
            res
        })
}