use worker::*;
use qrcode::QrCode;
use qrcode::render::svg;
use image::{Luma, ImageEncoder};
use base64::{Engine as _, engine::general_purpose};

// Validate that request comes from the app (same origin or allowed origin)
fn validate_origin(req: &Request, env: &Env) -> Result<()> {
    let url = req.url()?;
    let request_host = url.host_str().unwrap_or("");
    let request_scheme = url.scheme();
    
    // Check for API key in environment (optional, set ALLOWED_API_KEY in Cloudflare dashboard)
    if let Ok(api_key) = env.secret("ALLOWED_API_KEY") {
        if let Ok(Some(auth_header)) = req.headers().get("X-API-Key") {
            if auth_header.to_string() == api_key.to_string() {
                return Ok(()); // Valid API key, allow request
            }
        }
    }
    
    // Get Origin header (for cross-origin requests)
    let origin_header = req.headers().get("Origin")?;
    // Get Referer header (for same-origin requests)
    let referer_header = req.headers().get("Referer")?;
    
    // Check if request is from same origin
    let expected_origin = format!("{}://{}", request_scheme, request_host);
    
    // If Origin header is present, it must match the request host
    if let Some(ref origin) = origin_header {
        let origin_str = origin.to_string();
        // Allow same origin or localhost (for dev)
        if origin_str == expected_origin 
            || origin_str.contains(request_host)
            || origin_str.contains("localhost")
            || origin_str.contains("127.0.0.1") {
            return Ok(()); // Same origin or localhost, allow
        }
    }
    
    // Check Referer header
    if let Some(ref referer) = referer_header {
        let referer_str = referer.to_string();
        // Allow if referer matches host or is localhost (for dev)
        if referer_str.contains(request_host) 
            || referer_str.contains("localhost")
            || referer_str.contains("127.0.0.1") {
            return Ok(()); // Referer matches, allow
        }
    }
    
    // If neither Origin nor Referer is present, check if it's a direct API call
    // In dev mode (localhost), allow requests without Origin/Referer
    if request_host.contains("localhost") || request_host.contains("127.0.0.1") {
        return Ok(()); // Allow localhost requests in dev mode
    }
    
    // Block direct API calls (no Origin/Referer) unless they have API key
    if origin_header.is_none() && referer_header.is_none() {
        return Err(Error::RustError("Unauthorized: Direct API calls not allowed. Request must come from the app.".to_string()));
    }
    
    // Origin/Referer doesn't match, block request
    Err(Error::RustError("Unauthorized: Request must come from the app".to_string()))
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    
    Router::new()
        // API endpoint for QR generation
        .post_async("/api/generate", |mut req, ctx| async move {
            // Validate origin before processing
            validate_origin(&req, &ctx.env)?;
            
            let body: serde_json::Value = req.json().await?;
            
            let data = body["data"]
                .as_str()
                .ok_or("Missing 'data' field")?;
            
            let format = body["format"]
                .as_str()
                .unwrap_or("svg");
            
            generate_qr(data, format).await
        })
        
        // GET endpoint for URL-based generation
        .get_async("/api/qr", |req, ctx| async move {
            // Validate origin before processing
            validate_origin(&req, &ctx.env)?;
            
            let url = req.url()?;
            let query: std::collections::HashMap<String, String> = url
                .query_pairs()
                .into_owned()
                .collect();
            
            let data = query.get("data")
                .ok_or("Missing 'data' query parameter")?;
            
            let format = query.get("format")
                .map(|s| s.as_str())
                .unwrap_or("svg");
            
            generate_qr(data, format).await
        })
        
        // Capacity check endpoint (no origin validation needed - just checking)
        .post_async("/api/check-capacity", |mut req, _ctx| async move {
            let body: serde_json::Value = req.json().await?;
            
            let data = body["data"]
                .as_str()
                .ok_or("Missing 'data' field")?;
            
            let (_is_valid, capacity_info) = check_qr_capacity(data);
            
            Response::from_json(&capacity_info)
        })
        
        // Health check
        .get("/api/health", |_, _| {
            Response::from_json(&serde_json::json!({
                "status": "healthy",
                "service": "QR Generator WASM",
                "timestamp": Date::now().as_millis()
            }))
        })
        
        .run(req, env)
        .await
}

// Check QR code capacity before generation
// Actually attempts to create the QR code to verify it will work
// Returns (is_valid, capacity_info) where capacity_info includes:
// - byte_count: actual bytes used
// - max_capacity: maximum bytes for QR code (version 40, error correction L)
// - characters: character count
// - is_within_limit: whether it fits (VERIFIED by actually creating QR code)
fn check_qr_capacity(data: &str) -> (bool, serde_json::Value) {
    // QR Code Version 40 (maximum) with Error Correction Level L (lowest) = 2,953 bytes max
    // This is the theoretical maximum, but we'll verify by actually trying to create it
    const MAX_CAPACITY_BYTES: usize = 2953;
    
    let byte_count = data.as_bytes().len();
    let char_count = data.chars().count();
    
    // Actually try to create the QR code to verify it will work
    // This is the most reliable way to check capacity
    let qr_result = QrCode::new(data.as_bytes());
    let is_within_limit = qr_result.is_ok();
    
    // Calculate capacity info based on actual test result
    // Keep calculations simple and consistent to avoid weird jumps
    let (actual_max_capacity, bytes_over, bytes_remaining) = if !is_within_limit {
        // Generation failed - we're definitely over
        // To find how much over, we need to find the actual limit
        // Do a quick binary search (limited iterations for performance)
        let mut low = 0;
        let mut high = byte_count.min(MAX_CAPACITY_BYTES);
        let mut found_limit = 0;
        
        // Binary search with limited iterations (max 15 for performance)
        for _ in 0..15 {
            if low >= high {
                break;
            }
            let mid = (low + high + 1) / 2;
            let test_bytes: Vec<u8> = data.as_bytes().iter().take(mid).cloned().collect();
            
            if QrCode::new(&test_bytes).is_ok() {
                found_limit = mid;
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        
        // If we found a limit, use it; otherwise use conservative estimate
        let limit = if found_limit > 0 {
            found_limit
        } else {
            // Fallback: estimate limit is 90% of current (conservative)
            (byte_count * 9 / 10).max(1)
        };
        
        let over = byte_count.saturating_sub(limit);
        (limit, over, 0)
    } else {
        // Generation succeeded - find approximate max by testing larger sizes
        let mut test_size = byte_count;
        let mut found_limit = byte_count;
        
        // Test in larger increments first, then smaller (for performance)
        let increments = [200, 100, 50, 25, 10, 5, 1];
        for &inc in &increments {
            while test_size + inc <= MAX_CAPACITY_BYTES && test_size < byte_count + 1000 {
                test_size += inc;
                let test_bytes: Vec<u8> = data.as_bytes().iter()
                    .cycle()
                    .take(test_size)
                    .cloned()
                    .collect();
                
                if QrCode::new(&test_bytes).is_ok() {
                    found_limit = test_size;
                } else {
                    break;
                }
            }
        }
        
        let remaining = found_limit.saturating_sub(byte_count);
        (found_limit, 0, remaining)
    };
    
    // Calculate percentage based on the actual limit we found
    let percentage_used = if actual_max_capacity > 0 {
        ((byte_count as f64 / actual_max_capacity as f64) * 100.0).round() as u32
    } else {
        100
    };
    
    let capacity_info = serde_json::json!({
        "byte_count": byte_count,
        "char_count": char_count,
        "max_capacity_bytes": actual_max_capacity,  // Use actual tested limit
        "theoretical_max": MAX_CAPACITY_BYTES,     // Keep theoretical for reference
        "is_within_limit": is_within_limit,
        "bytes_over": bytes_over,
        "bytes_remaining": bytes_remaining,
        "percentage_used": percentage_used,
        "verified": true  // Indicates we actually tested QR code creation
    });
    
    (is_within_limit, capacity_info)
}

async fn generate_qr(data: &str, format: &str) -> Result<Response> {
    // Generate QR code - the capacity check already verified this will work
    // But we check again here as a safety measure
    let code = QrCode::new(data.as_bytes())
        .map_err(|e| {
            // If generation fails, provide helpful error message
            let byte_count = data.as_bytes().len();
            Error::RustError(format!(
                "QR generation failed: {}. Data size: {} bytes. Please reduce the data size.",
                e, byte_count
            ))
        })?;
    
    match format {
        "svg" => {
            let svg_string = code
                .render::<svg::Color>()
                .min_dimensions(300, 300)
                .dark_color(svg::Color("#000000"))
                .light_color(svg::Color("#ffffff"))
                .build();
            
            let headers = Headers::new();
            headers.set("Content-Type", "image/svg+xml")?;
            headers.set("Cache-Control", "public, max-age=3600")?;
            Ok(Response::ok(svg_string)?.with_headers(headers))
        },
        
        "png" => {
            let img = code.render::<Luma<u8>>()
                .min_dimensions(400, 400)
                .build();
            
            let mut png_bytes = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
            encoder.write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::L8,
            ).map_err(|e| Error::RustError(format!("PNG encoding failed: {}", e)))?;
            
            let base64_png = general_purpose::STANDARD.encode(&png_bytes);
            
            Response::from_json(&serde_json::json!({
                "format": "png",
                "data_url": format!("data:image/png;base64,{}", base64_png),
                "size_bytes": png_bytes.len(),
            }))
        },
        
        _ => Response::error("Invalid format. Use 'svg' or 'png'", 400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrcode::QrCode;

    #[test]
    fn test_qr_code_generation_basic() {
        let data = "https://example.com";
        let code = QrCode::new(data.as_bytes());
        assert!(code.is_ok());
    }

    #[test]
    fn test_qr_code_generation_empty_string() {
        let data = "";
        let code = QrCode::new(data.as_bytes());
        // Empty string should still generate a valid QR code
        assert!(code.is_ok());
    }

    #[test]
    fn test_qr_code_generation_long_string() {
        let data = "a".repeat(1000);
        let code = QrCode::new(data.as_bytes());
        assert!(code.is_ok());
    }

    #[test]
    fn test_qr_code_svg_rendering() {
        let data = "test";
        let code = QrCode::new(data.as_bytes()).unwrap();
        let svg_string = code
            .render::<svg::Color>()
            .min_dimensions(300, 300)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();
        
        // SVG output should contain SVG elements (may have whitespace before <svg>)
        assert!(svg_string.trim().starts_with("<svg") || svg_string.contains("<svg"));
        assert!(svg_string.contains("xmlns") || svg_string.contains("svg"));
        assert!(!svg_string.is_empty());
    }

    #[test]
    fn test_qr_code_png_rendering() {
        let data = "test";
        let code = QrCode::new(data.as_bytes()).unwrap();
        let img = code.render::<Luma<u8>>()
            .min_dimensions(400, 400)
            .build();
        
        assert!(img.width() >= 400);
        assert!(img.height() >= 400);
    }

    #[test]
    fn test_special_characters() {
        let test_cases = vec![
            "Hello, World!",
            "测试",
            "🚀 QR Code",
            "https://example.com?q=test&lang=en",
            "Line 1\nLine 2",
        ];

        for data in test_cases {
            let code = QrCode::new(data.as_bytes());
            assert!(code.is_ok(), "Failed to generate QR code for: {}", data);
        }
    }
}

