use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::client::conn::http1;
use hyper::Request;
use hyper_util::rt::TokioIo;
use std::path::Path;
use tokio::net::UnixStream;
use tracing::debug;

use crate::error::AppError;

/// Shared low-level HTTP client over Unix domain socket using Hyper 1.0 and TokioIo.
pub async fn send_unix_socket_request(
    socket_path: &Path,
    method: &str,
    path_and_query: &str,
    body_bytes: Option<Vec<u8>>,
    engine_name: &str,
) -> Result<(u16, Vec<u8>), AppError> {
    use std::os::unix::fs::FileTypeExt;

    let meta = std::fs::metadata(socket_path).map_err(|e| {
        AppError::InternalError(format!(
            "{} socket inaccessible at {:?}: {}",
            engine_name, socket_path, e
        ))
    })?;

    if !meta.file_type().is_socket() {
        return Err(AppError::InternalError(format!(
            "Path {:?} exists but is not a Unix domain socket",
            socket_path
        )));
    }

    let stream = UnixStream::connect(socket_path).await.map_err(|e| {
        AppError::InternalError(format!(
            "Failed to connect to {} socket at {:?}: {}",
            engine_name, socket_path, e
        ))
    })?;

    let io = TokioIo::new(stream);

    let handshake_fut = http1::handshake(io);
    let (mut sender, conn) = tokio::time::timeout(std::time::Duration::from_secs(5), handshake_fut)
        .await
        .map_err(|_| {
            AppError::InternalError(format!(
                "{} HTTP handshake timed out after 5s",
                engine_name
            ))
        })?
        .map_err(|e| {
            AppError::InternalError(format!("{} HTTP handshake failed: {}", engine_name, e))
        })?;

    let engine_name_owned = engine_name.to_string();
    tokio::spawn(async move {
        if let Err(err) = conn.await {
            debug!("{} connection closed: {:?}", engine_name_owned, err);
        }
    });

    let method_obj = match method {
        "POST" => hyper::Method::POST,
        "DELETE" => hyper::Method::DELETE,
        "PUT" => hyper::Method::PUT,
        _ => hyper::Method::GET,
    };

    let req_body = match body_bytes {
        Some(vec) => Full::new(Bytes::from(vec)),
        None => Full::new(Bytes::new()),
    };

    let req = Request::builder()
        .method(method_obj)
        .uri(path_and_query)
        .header("Host", "localhost")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body(req_body)
        .map_err(|e| {
            AppError::InternalError(format!(
                "Failed to build {} HTTP request: {}",
                engine_name, e
            ))
        })?;

    let response_fut = sender.send_request(req);
    let res = tokio::time::timeout(std::time::Duration::from_secs(5), response_fut)
        .await
        .map_err(|_| {
            AppError::InternalError(format!(
                "{} API request timed out after 5s",
                engine_name
            ))
        })?
        .map_err(|e| {
            AppError::InternalError(format!("{} API request error: {}", engine_name, e))
        })?;

    let status = res.status().as_u16();

    let collect_fut = res.into_body().collect();
    let collected = tokio::time::timeout(std::time::Duration::from_secs(5), collect_fut)
        .await
        .map_err(|_| {
            AppError::InternalError(format!(
                "Reading {} response body timed out after 5s",
                engine_name
            ))
        })?
        .map_err(|e| {
            AppError::InternalError(format!(
                "Failed to read {} response body: {}",
                engine_name, e
            ))
        })?;

    let bytes = collected.to_bytes().to_vec();
    Ok((status, bytes))
}
