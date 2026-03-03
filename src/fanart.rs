use rs_plugin_common_interfaces::domain::external_images::ImageType;
use serde::Deserialize;

// --- Serde structs for FanArt JSON responses ---

#[derive(Debug, Deserialize, Default)]
pub struct FanartMovieResponse {
    #[serde(default)]
    pub movieposter: Vec<FanartImage>,
    #[serde(default)]
    pub moviebackground: Vec<FanartImage>,
    #[serde(default)]
    pub hdmovielogo: Vec<FanartImage>,
    #[serde(default)]
    pub hdmovieclearart: Vec<FanartImage>,
    #[serde(default)]
    pub moviebanner: Vec<FanartImage>,
    #[serde(default)]
    pub moviethumb: Vec<FanartImage>,
    #[serde(default)]
    pub moviedisc: Vec<FanartImage>,
    #[serde(default)]
    pub movielogo: Vec<FanartImage>,
    #[serde(default)]
    pub movieart: Vec<FanartImage>,
}

#[derive(Debug, Deserialize, Default)]
pub struct FanartTvResponse {
    #[serde(default)]
    pub hdtvlogo: Vec<FanartImage>,
    #[serde(default)]
    pub clearlogo: Vec<FanartImage>,
    #[serde(default)]
    pub hdclearart: Vec<FanartImage>,
    #[serde(default)]
    pub showbackground: Vec<FanartImage>,
    #[serde(default)]
    pub tvthumb: Vec<FanartImage>,
    #[serde(default)]
    pub seasonposter: Vec<FanartImage>,
    #[serde(default)]
    pub tvbanner: Vec<FanartImage>,
    #[serde(default)]
    pub characterart: Vec<FanartImage>,
    #[serde(default)]
    pub seasonbanner: Vec<FanartImage>,
    #[serde(default)]
    pub tvposter: Vec<FanartImage>,
    #[serde(default)]
    pub seasonthumb: Vec<FanartImage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FanartImage {
    #[allow(dead_code)]
    pub id: Option<String>,
    pub url: Option<String>,
    pub lang: Option<String>,
    pub likes: Option<String>,
}

// --- Internal flattened image entry ---

#[derive(Debug, Clone)]
pub struct FanartImageEntry {
    pub url: String,
    pub image_type: ImageType,
    pub lang: Option<String>,
    pub likes: Option<String>,
}

// --- URL builders ---

pub fn build_movie_url(api_key: &str, id: &str) -> String {
    format!("https://webservice.fanart.tv/v3.2/movies/{id}?api_key={api_key}")
}

pub fn build_tv_url(api_key: &str, tvdb_id: u64) -> String {
    format!("https://webservice.fanart.tv/v3.2/tv/{tvdb_id}?api_key={api_key}")
}

// --- JSON parsers ---

pub fn parse_movie_response(json: &str) -> Option<Vec<FanartImageEntry>> {
    let response: FanartMovieResponse = serde_json::from_str(json).ok()?;
    Some(flatten_movie_images(&response))
}

pub fn parse_tv_response(json: &str) -> Option<Vec<FanartImageEntry>> {
    let response: FanartTvResponse = serde_json::from_str(json).ok()?;
    Some(flatten_tv_images(&response))
}

fn flatten_images(images: &[FanartImage], image_type: ImageType) -> Vec<FanartImageEntry> {
    images
        .iter()
        .filter_map(|img| {
            let url = img.url.as_ref()?.trim();
            if url.is_empty() {
                return None;
            }
            Some(FanartImageEntry {
                url: url.to_string(),
                image_type: image_type.clone(),
                lang: img.lang.clone().filter(|l| !l.is_empty()),
                likes: img.likes.clone(),
            })
        })
        .collect()
}

fn flatten_movie_images(response: &FanartMovieResponse) -> Vec<FanartImageEntry> {
    let mut entries = Vec::new();
    entries.extend(flatten_images(&response.movieposter, ImageType::Poster));
    entries.extend(flatten_images(
        &response.moviebackground,
        ImageType::Background,
    ));
    entries.extend(flatten_images(
        &response.hdmovielogo,
        ImageType::ClearLogo,
    ));
    entries.extend(flatten_images(
        &response.movielogo,
        ImageType::ClearLogo,
    ));
    entries.extend(flatten_images(
        &response.hdmovieclearart,
        ImageType::ClearArt,
    ));
    entries.extend(flatten_images(&response.moviethumb, ImageType::Card));
    entries.extend(flatten_images(
        &response.moviebanner,
        ImageType::Custom("banner".to_string()),
    ));
    entries.extend(flatten_images(
        &response.moviedisc,
        ImageType::Custom("disc".to_string()),
    ));
    entries.extend(flatten_images(
        &response.movieart,
        ImageType::ClearArt,
    ));
    entries
}

fn flatten_tv_images(response: &FanartTvResponse) -> Vec<FanartImageEntry> {
    let mut entries = Vec::new();
    entries.extend(flatten_images(&response.tvposter, ImageType::Poster));
    entries.extend(flatten_images(&response.seasonposter, ImageType::Poster));
    entries.extend(flatten_images(
        &response.showbackground,
        ImageType::Background,
    ));
    entries.extend(flatten_images(&response.hdtvlogo, ImageType::ClearLogo));
    entries.extend(flatten_images(&response.clearlogo, ImageType::ClearLogo));
    entries.extend(flatten_images(&response.hdclearart, ImageType::ClearArt));
    entries.extend(flatten_images(&response.tvthumb, ImageType::Card));
    entries.extend(flatten_images(&response.seasonthumb, ImageType::Card));
    entries.extend(flatten_images(
        &response.tvbanner,
        ImageType::Custom("banner".to_string()),
    ));
    entries.extend(flatten_images(
        &response.seasonbanner,
        ImageType::Custom("banner".to_string()),
    ));
    entries.extend(flatten_images(
        &response.characterart,
        ImageType::Custom("characterart".to_string()),
    ));
    entries
}

// --- ID parsing ---

/// Parse a movie ID for FanArt. Accepts:
/// - `tmdb:550` → `"550"`
/// - `imdb:tt0137523` → `"tt0137523"`
/// - Raw number `"550"` → `"550"`
/// - Raw IMDB `"tt0137523"` → `"tt0137523"`
pub fn parse_fanart_movie_id(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();

    if let Some(id_str) = lower.strip_prefix("tmdb:") {
        let id_str = id_str.trim();
        if !id_str.is_empty() && id_str.chars().all(|c| c.is_ascii_digit()) {
            return Some(id_str.to_string());
        }
    }

    if let Some(id_str) = lower.strip_prefix("tmdb-movie:") {
        let id_str = id_str.trim();
        if !id_str.is_empty() && id_str.chars().all(|c| c.is_ascii_digit()) {
            return Some(id_str.to_string());
        }
    }

    if let Some(id_str) = lower.strip_prefix("imdb:") {
        let id_str = id_str.trim();
        if !id_str.is_empty() {
            return Some(id_str.to_string());
        }
    }

    // Raw number (TMDB ID)
    if trimmed.chars().all(|c| c.is_ascii_digit()) && !trimmed.is_empty() {
        return Some(trimmed.to_string());
    }

    // Raw IMDB ID (tt followed by digits)
    if lower.starts_with("tt") && lower[2..].chars().all(|c| c.is_ascii_digit()) && lower.len() > 2
    {
        return Some(trimmed.to_string());
    }

    None
}

/// Parse a TV ID for FanArt (TVDB ID). Accepts:
/// - `tvdb:81189` → `81189`
/// - Raw number `"81189"` → `81189`
pub fn parse_fanart_tv_id(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();

    if let Some(id_str) = lower.strip_prefix("tvdb:") {
        return id_str.trim().parse::<u64>().ok();
    }

    // Raw number
    trimmed.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_movie_url_basic() {
        let url = build_movie_url("test_key", "550");
        assert_eq!(
            url,
            "https://webservice.fanart.tv/v3.2/movies/550?api_key=test_key"
        );
    }

    #[test]
    fn build_movie_url_imdb() {
        let url = build_movie_url("test_key", "tt0137523");
        assert_eq!(
            url,
            "https://webservice.fanart.tv/v3.2/movies/tt0137523?api_key=test_key"
        );
    }

    #[test]
    fn build_tv_url_basic() {
        let url = build_tv_url("test_key", 81189);
        assert_eq!(
            url,
            "https://webservice.fanart.tv/v3.2/tv/81189?api_key=test_key"
        );
    }

    #[test]
    fn parse_fanart_movie_id_tmdb_prefix() {
        assert_eq!(
            parse_fanart_movie_id("tmdb:550"),
            Some("550".to_string())
        );
        assert_eq!(
            parse_fanart_movie_id("TMDB:550"),
            Some("550".to_string())
        );
        assert_eq!(
            parse_fanart_movie_id("tmdb-movie:550"),
            Some("550".to_string())
        );
    }

    #[test]
    fn parse_fanart_movie_id_imdb_prefix() {
        assert_eq!(
            parse_fanart_movie_id("imdb:tt0137523"),
            Some("tt0137523".to_string())
        );
    }

    #[test]
    fn parse_fanart_movie_id_raw_number() {
        assert_eq!(
            parse_fanart_movie_id("550"),
            Some("550".to_string())
        );
    }

    #[test]
    fn parse_fanart_movie_id_raw_imdb() {
        assert_eq!(
            parse_fanart_movie_id("tt0137523"),
            Some("tt0137523".to_string())
        );
    }

    #[test]
    fn parse_fanart_movie_id_invalid() {
        assert_eq!(parse_fanart_movie_id(""), None);
        assert_eq!(parse_fanart_movie_id("  "), None);
        assert_eq!(parse_fanart_movie_id("Fight Club"), None);
        assert_eq!(parse_fanart_movie_id("tmdb:abc"), None);
    }

    #[test]
    fn parse_fanart_tv_id_tvdb_prefix() {
        assert_eq!(parse_fanart_tv_id("tvdb:81189"), Some(81189));
        assert_eq!(parse_fanart_tv_id("TVDB:81189"), Some(81189));
    }

    #[test]
    fn parse_fanart_tv_id_raw_number() {
        assert_eq!(parse_fanart_tv_id("81189"), Some(81189));
    }

    #[test]
    fn parse_fanart_tv_id_invalid() {
        assert_eq!(parse_fanart_tv_id(""), None);
        assert_eq!(parse_fanart_tv_id("  "), None);
        assert_eq!(parse_fanart_tv_id("Breaking Bad"), None);
    }

    #[test]
    fn parse_movie_response_basic() {
        let json = r#"{
            "movieposter": [
                {"id": "1", "url": "https://assets.fanart.tv/fanart/movies/550/movieposter/poster1.jpg", "lang": "en", "likes": "5"},
                {"id": "2", "url": "https://assets.fanart.tv/fanart/movies/550/movieposter/poster2.jpg", "lang": "de", "likes": "2"}
            ],
            "moviebackground": [
                {"id": "3", "url": "https://assets.fanart.tv/fanart/movies/550/moviebackground/bg1.jpg", "lang": "", "likes": "10"}
            ],
            "hdmovielogo": [
                {"id": "4", "url": "https://assets.fanart.tv/fanart/movies/550/hdmovielogo/logo1.png", "lang": "en", "likes": "3"}
            ]
        }"#;

        let entries = parse_movie_response(json).expect("parse");
        assert_eq!(entries.len(), 4);

        assert_eq!(entries[0].image_type, ImageType::Poster);
        assert!(entries[0].url.contains("poster1.jpg"));
        assert_eq!(entries[0].lang, Some("en".to_string()));
        assert_eq!(entries[0].likes, Some("5".to_string()));

        assert_eq!(entries[1].image_type, ImageType::Poster);
        assert_eq!(entries[2].image_type, ImageType::Background);
        assert_eq!(entries[2].lang, None); // empty string filtered out
        assert_eq!(entries[3].image_type, ImageType::ClearLogo);
    }

    #[test]
    fn parse_tv_response_basic() {
        let json = r#"{
            "hdtvlogo": [
                {"id": "1", "url": "https://assets.fanart.tv/fanart/tv/81189/hdtvlogo/logo1.png", "lang": "en", "likes": "3"}
            ],
            "showbackground": [
                {"id": "2", "url": "https://assets.fanart.tv/fanart/tv/81189/showbackground/bg1.jpg", "lang": "en", "likes": "7"}
            ],
            "tvposter": [
                {"id": "3", "url": "https://assets.fanart.tv/fanart/tv/81189/tvposter/poster1.jpg", "lang": "en", "likes": "4"}
            ]
        }"#;

        let entries = parse_tv_response(json).expect("parse");
        assert_eq!(entries.len(), 3);

        // tvposter comes first in flatten order
        assert_eq!(entries[0].image_type, ImageType::Poster);
        assert_eq!(entries[1].image_type, ImageType::Background);
        assert_eq!(entries[2].image_type, ImageType::ClearLogo);
    }

    #[test]
    fn parse_movie_response_empty() {
        let json = r#"{}"#;
        let entries = parse_movie_response(json).expect("parse");
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_response_skips_empty_urls() {
        let json = r#"{
            "movieposter": [
                {"id": "1", "url": "", "lang": "en", "likes": "1"},
                {"id": "2", "url": "https://assets.fanart.tv/poster.jpg", "lang": "en", "likes": "2"}
            ]
        }"#;

        let entries = parse_movie_response(json).expect("parse");
        assert_eq!(entries.len(), 1);
        assert!(entries[0].url.contains("poster.jpg"));
    }
}
