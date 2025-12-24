use worker::*;
use qrcode::QrCode;
use qrcode::render::svg;
use image::Luma;
use base64::{Engine as _, engine::general_purpose};

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    
    Router::new()
        // API endpoint for QR generation
        .post_async("/api/generate", |mut req, _ctx| async move {
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
        .get_async("/api/qr", |req, _ctx| async move {
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
            
            Response::ok(svg_string).map(|r| {
                let mut headers = Headers::new();
                headers.set("Content-Type", "image/svg+xml")?;
                headers.set("Cache-Control", "public, max-age=3600")?;
                r.with_headers(headers)
            })?
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

