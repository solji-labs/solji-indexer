// src/api_router/ipfs.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use futures::future::join_all;
use serde_json::json;

use super::AppState;

/// IPFS upload request for text content
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct IpfsUploadRequest {
    /// Content to upload to IPFS
    pub content: String,
}

/// Batch IPFS content request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct IpfsBatchRequest {
    /// Array of IPFS hashes to retrieve
    pub hashes: Vec<String>,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/ipfs/upload", post(upload_to_ipfs))
        .route("/api/ipfs/batch", post(get_ipfs_batch_content))
        .route("/api/ipfs/{hash}", get(get_ipfs_content))
        .with_state(state)
}

/// Upload content to IPFS via Pinata
#[utoipa::path(
    post,
    path = "/api/ipfs/upload",
    request_body = IpfsUploadRequest,
    responses(
        (status = 200, description = "Content uploaded successfully", body = serde_json::Value),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Upload failed"),
    ),
    tag = "IPFS"
)]
pub async fn upload_to_ipfs(
    State(state): State<AppState>,
    Json(request): Json<IpfsUploadRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!(
        "[IPFS] Uploading content to IPFS, size: {}",
        request.content.len()
    );

    // Validate content
    if request.content.is_empty() {
        println!("[IPFS] Empty content provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.content.len() > 1024 * 1024 {
        // 1MB limit
        println!("[IPFS] Content too large: {} bytes", request.content.len());
        return Err(StatusCode::BAD_REQUEST);
    }
    // Upload to Pinata using ureq (synchronous)
    let content_clone = request.content.clone();
    let pinata_response: serde_json::Value = match tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::new();

        // Create multipart request manually since ureq has limited multipart support
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let mut body = format!("--{}\r\n", boundary);
        body.push_str(
            "Content-Disposition: form-data; name=\"file\"; filename=\"content.txt\"\r\n",
        );
        body.push_str("Content-Type: text/plain\r\n\r\n");
        body.push_str(&content_clone);
        body.push_str(&format!("\r\n--{}--\r\n", boundary));

        match agent
            .post(&state.config.pinata_api_url)
            .set(
                "Authorization",
                &format!("Bearer {}", state.config.pinata_jwt),
            )
            .set(
                "Content-Type",
                &format!("multipart/form-data; boundary={}", boundary),
            )
            .send_bytes(body.as_bytes())
        {
            Ok(resp) => {
                if resp.status() == 200 {
                    match resp.into_json::<serde_json::Value>() {
                        Ok(json) => Ok(json),
                        Err(e) => {
                            println!("[IPFS] Failed to parse Pinata response: {:?}", e);
                            Err("JSON parse error".to_string())
                        }
                    }
                } else {
                    println!("[IPFS] Pinata returned status: {}", resp.status());
                    Err(format!("HTTP {}", resp.status()))
                }
            }
            Err(e) => {
                println!("[IPFS] Failed to send request to Pinata: {:?}", e);
                Err(format!("Request error: {:?}", e))
            }
        }
    })
    .await
    {
        Ok(result) => match result {
            Ok(json) => json,
            Err(e) => {
                println!("[IPFS] Upload failed: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        Err(e) => {
            println!("[IPFS] Task join error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Some(ipfs_hash) = pinata_response.get("IpfsHash").and_then(|h| h.as_str()) {
        let url = format!("{}/{}", state.config.pinata_gateway, ipfs_hash);

        println!("[IPFS] Content uploaded successfully, hash: {}", ipfs_hash);

        // Convert CID to byte array for contract compatibility
        let content_hash_array = if ipfs_hash.starts_with("Qm") && ipfs_hash.len() == 46 {
            // CID v0 format: convert to byte array
            match bs58::decode(ipfs_hash).into_vec() {
                Ok(decoded) => {
                    if decoded.len() >= 34 {
                        // Skip first 2 bytes (multihash header) and take 32 bytes
                        let mut hash = [0u8; 32];
                        hash.copy_from_slice(&decoded[2..34]);
                        serde_json::Value::Array(
                            hash.iter()
                                .map(|&b| serde_json::Value::Number(b.into()))
                                .collect(),
                        )
                    } else {
                        serde_json::Value::String(ipfs_hash.to_string())
                    }
                }
                Err(_) => serde_json::Value::String(ipfs_hash.to_string()),
            }
        } else {
            serde_json::Value::String(ipfs_hash.to_string())
        };

        Ok(Json(json!({
            "hash": ipfs_hash,
            "content_hash": content_hash_array,
            "url": url,
            "size": request.content.len()
        })))
    } else {
        println!("[IPFS] Invalid response from Pinata: {:?}", pinata_response);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Get IPFS content by hash
#[utoipa::path(
    get,
    path = "/api/ipfs/{hash}",
    params(
        ("hash" = String, Path, description = "IPFS hash/CID"),
    ),
    responses(
        (status = 200, description = "Content retrieved", body = serde_json::Value),
        (status = 404, description = "Content not found"),
        (status = 500, description = "Retrieval failed"),
    ),
    tag = "IPFS"
)]
pub async fn get_ipfs_content(
    State(state): State<AppState>,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("[IPFS] Retrieving content for hash: {}", hash);

    // Validate hash format (basic validation)
    if hash.is_empty() || hash.len() < 10 {
        println!("[IPFS] Invalid hash format: {}", hash);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Try to fetch from IPFS gateway using ureq
    let gateway_url = format!("{}/{}", state.config.pinata_gateway, hash);
    let hash_clone = hash.clone();
    let gateway_url_clone = gateway_url.clone();

    let (content_type, content_bytes) = match tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::new();

        match agent
            .get(&gateway_url_clone)
            .timeout(std::time::Duration::from_secs(30))
            .call()
        {
            Ok(resp) => {
                if resp.status() == 200 {
                    // Get content type from headers
                    let content_type = resp
                        .header("content-type")
                        .unwrap_or("application/octet-stream")
                        .to_string();

                    // Get response body
                    use std::io::Read;
                    let mut reader = resp.into_reader();
                    let mut bytes = Vec::new();
                    match reader.read_to_end(&mut bytes) {
                        Ok(_) => Ok((content_type, bytes)),
                        Err(e) => {
                            println!("[IPFS] Failed to read response body: {:?}", e);
                            Err("Body read error".to_string())
                        }
                    }
                } else {
                    println!("[IPFS] Gateway returned status: {}", resp.status());
                    Err(format!("HTTP {}", resp.status()))
                }
            }
            Err(e) => {
                println!("[IPFS] Failed to fetch from gateway: {:?}", e);
                Err(format!("Request error: {:?}", e))
            }
        }
    })
    .await
    {
        Ok(result) => match result {
            Ok((content_type, bytes)) => (content_type, bytes),
            Err(e) => {
                println!("[IPFS] Fetch failed: {}", e);
                return if e.contains("HTTP 404") {
                    Err(StatusCode::NOT_FOUND)
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                };
            }
        },
        Err(e) => {
            println!("[IPFS] Task join error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Try to parse as JSON first, otherwise return as text
    let content = if content_type.contains("application/json") {
        match serde_json::from_slice(&content_bytes) {
            Ok(json_value) => json_value,
            Err(_) => {
                // If JSON parsing fails, return as string
                serde_json::Value::String(String::from_utf8_lossy(&content_bytes).to_string())
            }
        }
    } else {
        // For non-JSON content, return as string
        serde_json::Value::String(String::from_utf8_lossy(&content_bytes).to_string())
    };

    println!(
        "[IPFS] Content retrieved successfully, size: {} bytes",
        content_bytes.len()
    );

    let response = json!({
        "hash": hash,
        "content": content,
        "content_type": content_type,
        "size": content_bytes.len(),
        "gateway_url": gateway_url
    });

    Ok(Json(response))
}

/// Get multiple IPFS contents in batch
#[utoipa::path(
    post,
    path = "/api/ipfs/batch",
    request_body = IpfsBatchRequest,
    responses(
        (status = 200, description = "Batch content retrieved", body = serde_json::Value,
            example = json!({
                "contents": [
                    {
                        "hash": "QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
                        "content": "content t",
                        "content_type": "text/plain",
                        "size": 1234,
                        "gateway_url": "https://solji.mypinata.cloud/ipfs/QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
                    }
                ],
                "errors": []
            })
        ),
        (status = 400, description = "Invalid request - too many hashes or invalid format"),
        (status = 500, description = "Batch retrieval failed"),
    ),
    tag = "IPFS"
)]
pub async fn get_ipfs_batch_content(
    State(state): State<AppState>,
    Json(request): Json<IpfsBatchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("[IPFS] Batch retrieving {} contents", request.hashes.len());

    // Validate request
    if request.hashes.is_empty() {
        println!("[IPFS] Empty hash list provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.hashes.len() > 50 {
        println!("[IPFS] Too many hashes requested: {}", request.hashes.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate all hashes
    for hash in &request.hashes {
        if hash.is_empty() || hash.len() < 10 {
            println!("[IPFS] Invalid hash format: {}", hash);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let gateway = &state.config.pinata_gateway;

    // Create futures for concurrent requests using ureq
    let futures = request.hashes.into_iter().map(|hash| {
        let gateway_url = format!("{}/{}", gateway, hash);
        async move {
            let hash_for_result = hash.clone();
            let gateway_url_for_result = gateway_url.clone();

            // Use spawn_blocking for ureq synchronous calls
            match tokio::task::spawn_blocking(move || {
                let agent = ureq::Agent::new();

                match agent
                    .get(&gateway_url)
                    .timeout(std::time::Duration::from_secs(30))
                    .call()
                {
                    Ok(resp) => {
                        if resp.status() == 200 {
                            // Get content type from headers
                            let content_type = resp
                                .header("content-type")
                                .unwrap_or("application/octet-stream")
                                .to_string();

                            // Get response body
                            use std::io::Read;
                            let mut reader = resp.into_reader();
                            let mut bytes = Vec::new();
                            match reader.read_to_end(&mut bytes) {
                                Ok(_) => Ok((content_type, bytes)),
                                Err(e) => {
                                    println!(" [IPFS] Failed to read body for {}: {:?}", hash, e);
                                    Err((hash, format!("Body read error: {:?}", e)))
                                }
                            }
                        } else {
                            println!(" [IPFS] Gateway returned {} for {}", resp.status(), hash);
                            Err((hash, format!("HTTP {}", resp.status())))
                        }
                    }
                    Err(e) => {
                        println!(" [IPFS] Failed to fetch {}: {:?}", hash, e);
                        Err((hash, format!("Network error: {:?}", e)))
                    }
                }
            })
            .await
            {
                Ok(Ok((content_type, content_bytes))) => {
                    // Parse content
                    let content = if content_type.contains("application/json") {
                        match serde_json::from_slice(&content_bytes) {
                            Ok(json_value) => json_value,
                            Err(_) => serde_json::Value::String(
                                String::from_utf8_lossy(&content_bytes).to_string(),
                            ),
                        }
                    } else {
                        serde_json::Value::String(
                            String::from_utf8_lossy(&content_bytes).to_string(),
                        )
                    };

                    println!(
                        " [IPFS] Retrieved {} ({} bytes)",
                        hash_for_result,
                        content_bytes.len()
                    );

                    Ok(json!({
                        "hash": hash_for_result,
                        "content": content,
                        "content_type": content_type,
                        "size": content_bytes.len(),
                        "gateway_url": gateway_url_for_result
                    }))
                }
                Ok(Err((hash, error))) => Err((hash, error)),
                Err(e) => {
                    println!(" [IPFS] Task join error for {}: {:?}", hash_for_result, e);
                    Err((hash_for_result, format!("Task error: {:?}", e)))
                }
            }
        }
    });

    // Execute all requests concurrently
    let results = join_all(futures).await;

    // Separate successful results and errors
    let mut contents = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(content) => contents.push(content),
            Err((hash, error)) => errors.push(json!({
                "hash": hash,
                "error": error
            })),
        }
    }

    println!(
        " [IPFS] Batch retrieval completed: {} successful, {} errors",
        contents.len(),
        errors.len()
    );

    Ok(Json(json!({
        "contents": contents,
        "errors": errors
    })))
}
