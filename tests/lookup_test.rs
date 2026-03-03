use extism::*;
use rs_plugin_common_interfaces::{
    domain::external_images::ExternalImage,
    lookup::{
        RsLookupMetadataResults, RsLookupMovie, RsLookupQuery, RsLookupSerie, RsLookupWrapper,
    },
};

fn build_plugin() -> Plugin {
    let wasm = Wasm::file("target/wasm32-unknown-unknown/release/rs_plugin_fanart.wasm");
    let manifest = Manifest::new([wasm]).with_allowed_hosts(
        ["webservice.fanart.tv"]
            .iter()
            .map(|s| s.to_string()),
    );
    Plugin::new(&manifest, [], true).expect("Failed to create plugin")
}

fn call_lookup_images(plugin: &mut Plugin, input: &RsLookupWrapper) -> Vec<ExternalImage> {
    let input_str = serde_json::to_string(input).unwrap();
    let output = plugin
        .call::<&str, &[u8]>("lookup_metadata_images", &input_str)
        .expect("lookup_metadata_images call failed");
    serde_json::from_slice(output).expect("Failed to parse images output")
}

fn call_lookup_metadata(plugin: &mut Plugin, input: &RsLookupWrapper) -> RsLookupMetadataResults {
    let input_str = serde_json::to_string(input).unwrap();
    let output = plugin
        .call::<&str, &[u8]>("lookup_metadata", &input_str)
        .expect("lookup_metadata call failed");
    serde_json::from_slice(output).expect("Failed to parse metadata output")
}

#[test]
fn test_default_key_works() {
    let mut plugin = build_plugin();

    let input = RsLookupWrapper {
        query: RsLookupQuery::Movie(RsLookupMovie {
            name: Some("tmdb:550".to_string()),
            ids: None,
            page_key: None,
        }),
        credential: None,
        params: None,
    };

    let images = call_lookup_images(&mut plugin, &input);
    assert!(
        !images.is_empty(),
        "Expected images using default API key for tmdb:550"
    );
}

#[test]
fn test_movie_images_by_tmdb_id() {
    let mut plugin = build_plugin();

    let input = RsLookupWrapper {
        query: RsLookupQuery::Movie(RsLookupMovie {
            name: Some("tmdb:550".to_string()),
            ids: None,
            page_key: None,
        }),
        credential: None,
        params: None,
    };

    let images = call_lookup_images(&mut plugin, &input);
    assert!(
        !images.is_empty(),
        "Expected at least one image for Fight Club (tmdb:550)"
    );

    println!("Got {} images for tmdb:550", images.len());
    for img in &images {
        println!("  {:?}: {}", img.kind, img.url.url);
    }

    // Should have at least poster or background
    let has_poster = images
        .iter()
        .any(|i| i.kind == Some(rs_plugin_common_interfaces::domain::external_images::ImageType::Poster));
    let has_background = images
        .iter()
        .any(|i| i.kind == Some(rs_plugin_common_interfaces::domain::external_images::ImageType::Background));
    assert!(
        has_poster || has_background,
        "Expected at least a poster or background image"
    );
}

#[test]
fn test_tv_images_by_tvdb_id() {
    let mut plugin = build_plugin();

    let input = RsLookupWrapper {
        query: RsLookupQuery::Serie(RsLookupSerie {
            name: Some("tvdb:81189".to_string()),
            ids: None,
            page_key: None,
        }),
        credential: None,
        params: None,
    };

    let images = call_lookup_images(&mut plugin, &input);
    assert!(
        !images.is_empty(),
        "Expected at least one image for Breaking Bad (tvdb:81189)"
    );

    println!("Got {} images for tvdb:81189", images.len());
    for img in &images {
        println!("  {:?}: {}", img.kind, img.url.url);
    }
}

#[test]
fn test_movie_no_id_returns_empty() {
    let mut plugin = build_plugin();

    let input = RsLookupWrapper {
        query: RsLookupQuery::Movie(RsLookupMovie {
            name: Some("Fight Club".to_string()),
            ids: None,
            page_key: None,
        }),
        credential: None,
        params: None,
    };

    let images = call_lookup_images(&mut plugin, &input);
    assert!(
        images.is_empty(),
        "Expected empty results for name-only query (FanArt has no search)"
    );
}

#[test]
fn test_serie_no_tvdb_returns_empty() {
    let mut plugin = build_plugin();

    let input = RsLookupWrapper {
        query: RsLookupQuery::Serie(RsLookupSerie {
            name: Some("Breaking Bad".to_string()),
            ids: None,
            page_key: None,
        }),
        credential: None,
        params: None,
    };

    let images = call_lookup_images(&mut plugin, &input);
    assert!(
        images.is_empty(),
        "Expected empty results for name-only serie query (FanArt needs TVDB ID)"
    );
}

#[test]
fn test_metadata_returns_empty() {
    let mut plugin = build_plugin();

    let input = RsLookupWrapper {
        query: RsLookupQuery::Movie(RsLookupMovie {
            name: Some("tmdb:550".to_string()),
            ids: None,
            page_key: None,
        }),
        credential: None,
        params: None,
    };

    let results = call_lookup_metadata(&mut plugin, &input);
    assert!(
        results.results.is_empty(),
        "FanArt should return empty metadata (image-only provider)"
    );
}
