use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::imageops::FilterType;
use image::ImageFormat;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE};
use scraper::{Html, Selector};
use std::io::Cursor;
use std::time::Duration;

const THUMBNAIL_MAX_WIDTH: u32 = 400;
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// Fetch the Open Graph preview image for a URL and return it as a base64-encoded JPEG thumbnail.
/// Returns `None` on any failure (network, parsing, missing OG image, etc.).
pub async fn fetch_og_image_base64(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .ok()?;

    // Fetch the page HTML with browser-like headers
    let response = client
        .get(url)
        .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9")
        .send()
        .await;

    let response = match response {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[OG] HTTP request failed for {}: {}", url, e);
            return None;
        }
    };

    if !response.status().is_success() {
        eprintln!("[OG] HTTP {} for {}", response.status(), url);
        return None;
    }

    let html_text = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[OG] Failed to read response body for {}: {}", url, e);
            return None;
        }
    };

    // Parse OG image URL from meta tags
    let image_url = match extract_og_image_url(&html_text, url) {
        Some(u) => {
            eprintln!("[OG] Found image URL: {}", u);
            u
        }
        None => {
            eprintln!("[OG] No og:image or twitter:image found for {}", url);
            return None;
        }
    };

    // Fetch the image
    let img_response = match client.get(&image_url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[OG] Failed to fetch image {}: {}", image_url, e);
            return None;
        }
    };

    if !img_response.status().is_success() {
        eprintln!("[OG] Image HTTP {} for {}", img_response.status(), image_url);
        return None;
    }

    let img_bytes = match img_response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[OG] Failed to read image bytes from {}: {}", image_url, e);
            return None;
        }
    };

    eprintln!("[OG] Downloaded image: {} bytes", img_bytes.len());

    // Decode, resize, and encode as JPEG
    let image = match image::load_from_memory(&img_bytes) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("[OG] Failed to decode image: {}", e);
            return None;
        }
    };

    let (w, h) = (image.width(), image.height());
    eprintln!("[OG] Image dimensions: {}x{}", w, h);

    let thumbnail = if w > THUMBNAIL_MAX_WIDTH {
        let ratio = THUMBNAIL_MAX_WIDTH as f32 / w as f32;
        let new_h = (h as f32 * ratio) as u32;
        image.resize(THUMBNAIL_MAX_WIDTH, new_h, FilterType::Lanczos3)
    } else {
        image
    };

    let mut jpeg_bytes = Vec::new();
    if let Err(e) = thumbnail.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageFormat::Jpeg) {
        eprintln!("[OG] Failed to encode JPEG: {}", e);
        return None;
    }

    eprintln!("[OG] Thumbnail encoded: {} bytes JPEG", jpeg_bytes.len());
    Some(BASE64.encode(&jpeg_bytes))
}

/// Extract the OG image URL from HTML meta tags.
/// Tries og:image first, then twitter:image as fallback.
/// Resolves relative URLs against the page URL.
fn extract_og_image_url(html: &str, page_url: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Try og:image
    if let Ok(og_selector) = Selector::parse(r#"meta[property="og:image"]"#) {
        if let Some(element) = document.select(&og_selector).next() {
            if let Some(content) = element.value().attr("content") {
                if !content.is_empty() {
                    return Some(resolve_url(content, page_url));
                }
            }
        }
    }

    // Fallback: twitter:image (name= attribute)
    if let Ok(selector) = Selector::parse(r#"meta[name="twitter:image"]"#) {
        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                if !content.is_empty() {
                    return Some(resolve_url(content, page_url));
                }
            }
        }
    }

    // Fallback: twitter:image (property= attribute)
    if let Ok(selector) = Selector::parse(r#"meta[property="twitter:image"]"#) {
        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                if !content.is_empty() {
                    return Some(resolve_url(content, page_url));
                }
            }
        }
    }

    None
}

/// Resolve a potentially relative URL against a base page URL.
fn resolve_url(image_url: &str, page_url: &str) -> String {
    if image_url.starts_with("http://") || image_url.starts_with("https://") {
        return image_url.to_string();
    }
    // Try to resolve relative URL
    if let Ok(base) = reqwest::Url::parse(page_url) {
        if let Ok(resolved) = base.join(image_url) {
            return resolved.to_string();
        }
    }
    image_url.to_string()
}
