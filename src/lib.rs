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
        if origin_str == expected_origin || origin_str.contains(request_host) {
            return Ok(()); // Same origin, allow
        }
    }
    
    // Check Referer header
    if let Some(ref referer) = referer_header {
        let referer_str = referer.to_string();
        if referer_str.contains(request_host) {
            return Ok(()); // Referer matches, allow
        }
    }
    
    // If neither Origin nor Referer is present, check if it's a direct API call
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

async fn generate_qr(data: &str, format: &str) -> Result<Response> {
    // Generate QR code
    let code = QrCode::new(data.as_bytes())
        .map_err(|e| Error::RustError(format!("QR generation failed: {}", e)))?;
    
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

