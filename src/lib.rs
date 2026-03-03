use extism_pdk::{http, log, plugin_fn, FnResult, HttpRequest, Json, LogLevel, WithReturnCode};
use std::collections::HashSet;

use rs_plugin_common_interfaces::{
    domain::external_images::ExternalImage,
    lookup::{
        RsLookupMetadataResults, RsLookupMovie, RsLookupQuery, RsLookupSerie, RsLookupWrapper,
    },
    CredentialType, PluginInformation, PluginType, RsRequest,
};

mod fanart;

use fanart::{
    build_movie_url, build_tv_url, parse_fanart_movie_id, parse_fanart_tv_id,
    parse_movie_response, parse_tv_response, FanartImageEntry,
};

const DEFAULT_API_KEY: &str = "a6eb2f1acb7b54550e498a9b37a574fa";

#[plugin_fn]
pub fn infos() -> FnResult<Json<PluginInformation>> {
    Ok(Json(PluginInformation {
        name: "fanart_images".into(),
        capabilities: vec![PluginType::LookupMetadata],
        version: 2,
        interface_version: 1,
        repo: Some("https://github.com/neckaros/rs-plugin-fanart".to_string()),
        publisher: "neckaros".into(),
        description: "Look up movie and TV show artwork from FanArt.tv".into(),
        credential_kind: Some(CredentialType::Token),
        settings: vec![],
        ..Default::default()
    }))
}

fn build_http_request(url: String) -> HttpRequest {
    let mut request = HttpRequest {
        url,
        headers: Default::default(),
        method: Some("GET".into()),
    };

    request
        .headers
        .insert("Accept".to_string(), "application/json".to_string());
    request.headers.insert(
        "User-Agent".to_string(),
        "rs-plugin-fanart/0.1 (+https://fanart.tv)".to_string(),
    );

    request
}

fn extract_api_key(lookup: &RsLookupWrapper) -> FnResult<String> {
    if let Some(key) = lookup
        .credential
        .as_ref()
        .and_then(|c| c.password.as_deref())
        .map(str::trim)
        .filter(|k| !k.is_empty())
    {
        return Ok(key.to_string());
    }

    Ok(DEFAULT_API_KEY.to_string())
}

fn execute_json_request(url: String) -> FnResult<String> {
    let request = build_http_request(url);
    let res = http::request::<Vec<u8>>(&request, None);

    match res {
        Ok(res) if res.status_code() >= 200 && res.status_code() < 300 => {
            Ok(String::from_utf8_lossy(&res.body()).to_string())
        }
        Ok(res) => {
            log!(
                LogLevel::Error,
                "FanArt HTTP error {}: {}",
                res.status_code(),
                String::from_utf8_lossy(&res.body())
            );
            Err(WithReturnCode::new(
                extism_pdk::Error::msg(format!("HTTP error: {}", res.status_code())),
                res.status_code() as i32,
            ))
        }
        Err(e) => {
            log!(LogLevel::Error, "FanArt request failed: {}", e);
            Err(WithReturnCode(e, 500))
        }
    }
}

/// Resolve a movie query to a FanArt-compatible ID string (TMDB or IMDB).
fn resolve_movie_id(movie: &RsLookupMovie) -> Option<String> {
    // Check name for ID patterns
    if let Some(name) = movie.name.as_deref() {
        if let Some(id) = parse_fanart_movie_id(name) {
            return Some(id);
        }
    }

    // Check ids.tmdb
    if let Some(ids) = movie.ids.as_ref() {
        if let Some(tmdb_id) = ids.tmdb {
            return Some(tmdb_id.to_string());
        }
        if let Some(ref imdb_id) = ids.imdb {
            let trimmed = imdb_id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Resolve a serie query to a TVDB ID.
fn resolve_serie_id(serie: &RsLookupSerie) -> Option<u64> {
    // Check name for tvdb:ID pattern
    if let Some(name) = serie.name.as_deref() {
        if let Some(id) = parse_fanart_tv_id(name) {
            return Some(id);
        }
    }

    // Check ids.tvdb
    if let Some(ids) = serie.ids.as_ref() {
        if let Some(tvdb_id) = ids.tvdb {
            return Some(tvdb_id);
        }
    }

    None
}

fn fanart_entry_to_image(entry: FanartImageEntry) -> ExternalImage {
    ExternalImage {
        kind: Some(entry.image_type),
        url: RsRequest {
            url: entry.url,
            ..Default::default()
        },
        lang: entry.lang,
        vote_count: entry.likes.as_deref().and_then(|l| l.parse::<i64>().ok()),
        ..Default::default()
    }
}

fn fetch_movie_images(api_key: &str, id: &str) -> FnResult<Vec<ExternalImage>> {
    let url = build_movie_url(api_key, id);
    let body = execute_json_request(url)?;
    let entries = parse_movie_response(&body).unwrap_or_default();
    Ok(entries.into_iter().map(fanart_entry_to_image).collect())
}

fn fetch_tv_images(api_key: &str, tvdb_id: u64) -> FnResult<Vec<ExternalImage>> {
    let url = build_tv_url(api_key, tvdb_id);
    let body = execute_json_request(url)?;
    let entries = parse_tv_response(&body).unwrap_or_default();
    Ok(entries.into_iter().map(fanart_entry_to_image).collect())
}

fn lookup_images(lookup: &RsLookupWrapper, api_key: &str) -> FnResult<Vec<ExternalImage>> {
    match &lookup.query {
        RsLookupQuery::Movie(movie) => match resolve_movie_id(movie) {
            Some(id) => fetch_movie_images(api_key, &id),
            None => Ok(vec![]),
        },
        RsLookupQuery::Serie(serie) => match resolve_serie_id(serie) {
            Some(tvdb_id) => fetch_tv_images(api_key, tvdb_id),
            None => Ok(vec![]),
        },
        _ => Ok(vec![]),
    }
}

#[plugin_fn]
pub fn lookup_metadata(
    Json(_lookup): Json<RsLookupWrapper>,
) -> FnResult<Json<RsLookupMetadataResults>> {
    // FanArt.tv is image-only — no metadata to return
    Ok(Json(RsLookupMetadataResults {
        results: vec![],
        next_page_key: None,
    }))
}

#[plugin_fn]
pub fn lookup_metadata_images(
    Json(lookup): Json<RsLookupWrapper>,
) -> FnResult<Json<Vec<ExternalImage>>> {
    let api_key = extract_api_key(&lookup)?;
    let images = lookup_images(&lookup, &api_key)?;
    Ok(Json(deduplicate_images(images)))
}

fn deduplicate_images(images: Vec<ExternalImage>) -> Vec<ExternalImage> {
    let mut seen_urls = HashSet::new();
    let mut deduped = Vec::new();

    for image in images {
        if seen_urls.insert(image.url.url.clone()) {
            deduped.push(image);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs_plugin_common_interfaces::domain::rs_ids::RsIds;

    #[test]
    fn extract_api_key_missing_returns_default() {
        let lookup = RsLookupWrapper {
            query: RsLookupQuery::Movie(Default::default()),
            credential: None,
            params: None,
        };

        let key = extract_api_key(&lookup).expect("should return default key");
        assert_eq!(key, DEFAULT_API_KEY);
    }

    #[test]
    fn extract_api_key_empty_returns_default() {
        let lookup = RsLookupWrapper {
            query: RsLookupQuery::Movie(Default::default()),
            credential: Some(rs_plugin_common_interfaces::PluginCredential {
                kind: CredentialType::Token,
                password: Some("  ".to_string()),
                ..Default::default()
            }),
            params: None,
        };

        let key = extract_api_key(&lookup).expect("should return default key");
        assert_eq!(key, DEFAULT_API_KEY);
    }

    #[test]
    fn extract_api_key_present() {
        let lookup = RsLookupWrapper {
            query: RsLookupQuery::Movie(Default::default()),
            credential: Some(rs_plugin_common_interfaces::PluginCredential {
                kind: CredentialType::Token,
                password: Some("my_api_key".to_string()),
                ..Default::default()
            }),
            params: None,
        };

        let key = extract_api_key(&lookup).expect("should extract key");
        assert_eq!(key, "my_api_key");
    }

    #[test]
    fn resolve_movie_id_from_name_tmdb() {
        let movie = RsLookupMovie {
            name: Some("tmdb:550".to_string()),
            ids: None,
            page_key: None,
        };
        assert_eq!(resolve_movie_id(&movie), Some("550".to_string()));
    }

    #[test]
    fn resolve_movie_id_from_name_imdb() {
        let movie = RsLookupMovie {
            name: Some("imdb:tt0137523".to_string()),
            ids: None,
            page_key: None,
        };
        assert_eq!(resolve_movie_id(&movie), Some("tt0137523".to_string()));
    }

    #[test]
    fn resolve_movie_id_from_ids_tmdb() {
        let movie = RsLookupMovie {
            name: Some("Fight Club".to_string()),
            ids: Some(RsIds {
                tmdb: Some(550),
                ..Default::default()
            }),
            page_key: None,
        };
        assert_eq!(resolve_movie_id(&movie), Some("550".to_string()));
    }

    #[test]
    fn resolve_movie_id_from_ids_imdb() {
        let movie = RsLookupMovie {
            name: Some("Fight Club".to_string()),
            ids: Some(RsIds {
                imdb: Some("tt0137523".to_string()),
                ..Default::default()
            }),
            page_key: None,
        };
        assert_eq!(resolve_movie_id(&movie), Some("tt0137523".to_string()));
    }

    #[test]
    fn resolve_movie_id_no_id_returns_none() {
        let movie = RsLookupMovie {
            name: Some("Fight Club".to_string()),
            ids: None,
            page_key: None,
        };
        assert_eq!(resolve_movie_id(&movie), None);
    }

    #[test]
    fn resolve_serie_id_from_name_tvdb() {
        let serie = RsLookupSerie {
            name: Some("tvdb:81189".to_string()),
            ids: None,
            page_key: None,
        };
        assert_eq!(resolve_serie_id(&serie), Some(81189));
    }

    #[test]
    fn resolve_serie_id_from_ids_tvdb() {
        let serie = RsLookupSerie {
            name: Some("Breaking Bad".to_string()),
            ids: Some(RsIds {
                tvdb: Some(81189),
                ..Default::default()
            }),
            page_key: None,
        };
        assert_eq!(resolve_serie_id(&serie), Some(81189));
    }

    #[test]
    fn resolve_serie_id_no_id_returns_none() {
        let serie = RsLookupSerie {
            name: Some("Breaking Bad".to_string()),
            ids: None,
            page_key: None,
        };
        assert_eq!(resolve_serie_id(&serie), None);
    }

    #[test]
    fn lookup_non_movie_serie_returns_empty() {
        let lookup = RsLookupWrapper {
            query: RsLookupQuery::Book(Default::default()),
            credential: None,
            params: None,
        };

        let images = lookup_images(&lookup, "test_key").expect("should succeed");
        assert!(images.is_empty());
    }

    #[test]
    fn deduplicate_images_by_url() {
        let images = vec![
            ExternalImage {
                url: RsRequest {
                    url: "https://a.com/1.jpg".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ExternalImage {
                url: RsRequest {
                    url: "https://a.com/1.jpg".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let deduped = deduplicate_images(images);
        assert_eq!(deduped.len(), 1);
    }

    #[test]
    fn fanart_entry_converts_to_external_image() {
        use rs_plugin_common_interfaces::domain::external_images::ImageType;

        let entry = FanartImageEntry {
            url: "https://assets.fanart.tv/poster.jpg".to_string(),
            image_type: ImageType::Poster,
            lang: Some("en".to_string()),
            likes: Some("5".to_string()),
        };

        let image = fanart_entry_to_image(entry);
        assert_eq!(image.kind, Some(ImageType::Poster));
        assert_eq!(image.url.url, "https://assets.fanart.tv/poster.jpg");
        assert_eq!(image.lang, Some("en".to_string()));
        assert_eq!(image.vote_count, Some(5));
    }
}
