//! Remote metadata providers. Calls are explicit and results are written to
//! `MetadataCache`; ordinary library scans never perform network requests.

use cl_core::{Game, GameMetadata, SystemId};
use serde_json::Value;
use std::path::{Path, PathBuf};

const BASE: &str = "https://titlescreen.org/api/v1";
const USER_AGENT: &str = concat!("console-launcher/", env!("CARGO_PKG_VERSION"));

fn encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn platform(system: SystemId) -> &'static str {
    use SystemId::*;
    match system {
        Nes => "nes",
        Snes => "snes",
        N64 => "n64",
        Gb => "gb",
        Gbc => "gbc",
        Gba => "gba",
        Nds => "nds",
        MasterSystem => "sms",
        MegaDrive => "genesis",
        Psx => "ps1",
        Ps2 => "ps2",
        Psp => "psp",
        GameCube => "gc",
        Wii => "wii",
        WiiU => "wiiu",
    }
}

fn get(url: &str) -> Result<Value, String> {
    ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .call()
        .map_err(|error| error.to_string())?
        .body_mut()
        .read_json::<Value>()
        .map_err(|error| error.to_string())
}

fn normalise(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn title_from_work(work: &Value) -> Option<&str> {
    work.get("title").and_then(Value::as_str)
}

fn link_matches(work: &Value, serial: &str) -> bool {
    work.get("links")
        .and_then(Value::as_array)
        .is_some_and(|links| {
            links.iter().any(|link| {
                link.get("kind").and_then(Value::as_str) == Some("gametdb")
                    && link
                        .get("value")
                        .and_then(Value::as_str)
                        .is_some_and(|value| serial_matches(value, serial))
            })
        })
}

fn gametdb_ids(work: &Value) -> impl Iterator<Item = &str> {
    work.get("links")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|link| link.get("kind").and_then(Value::as_str) == Some("gametdb"))
        .filter_map(|link| link.get("value").and_then(Value::as_str))
}

fn gametdb_region(serial: Option<&str>) -> &'static str {
    match serial
        .and_then(|value| value.as_bytes().get(3))
        .map(u8::to_ascii_uppercase)
    {
        Some(b'E') => "US",
        Some(b'J') => "JA",
        Some(b'K') => "KO",
        Some(b'R') => "RU",
        Some(b'W') => "ZH",
        _ => "EN",
    }
}

fn release_region(release: Option<&Value>) -> Option<&'static str> {
    match release?
        .get("region")?
        .as_str()?
        .to_ascii_uppercase()
        .as_str()
    {
        "US" | "USA" | "NA" => Some("US"),
        "JP" | "JA" | "JAPAN" => Some("JA"),
        "KR" | "KO" | "KOREA" => Some("KO"),
        "RU" | "RUSSIA" => Some("RU"),
        "CN" | "CH" | "ZH" | "CHINA" | "TW" | "TAIWAN" => Some("ZH"),
        "AU" | "AUSTRALIA" => Some("AU"),
        "EU" | "EUR" | "EUROPE" | "PAL" => Some("EN"),
        _ => None,
    }
}

fn release_serial(release: Option<&Value>) -> Option<&str> {
    release?.get("serial")?.as_str()
}

fn serial_matches(candidate: &str, serial: &str) -> bool {
    let candidate_prefix = (candidate.len() == 6).then(|| candidate.get(..4)).flatten();
    candidate.eq_ignore_ascii_case(serial)
        || (serial.len() == 4
            && candidate_prefix.is_some_and(|prefix| prefix.eq_ignore_ascii_case(serial)))
        || serial
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|part| {
                part.eq_ignore_ascii_case(candidate)
                    || (part.len() == 4
                        && candidate_prefix.is_some_and(|prefix| prefix.eq_ignore_ascii_case(part)))
            })
}

fn gametdb_id<'a>(
    work: &'a Value,
    system: SystemId,
    release: Option<&Value>,
    serial: Option<&str>,
) -> Option<&'a str> {
    let required_len = if system == SystemId::Nds { 4 } else { 6 };
    let ids = gametdb_ids(work)
        .filter(|id| {
            id.len() == required_len && id.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
        .collect::<Vec<_>>();
    for identity in [serial, release_serial(release)].into_iter().flatten() {
        if let Some(id) = ids.iter().copied().find(|id| serial_matches(id, identity)) {
            return Some(id);
        }
    }
    if let Some(region) = release_region(release) {
        if let Some(id) = ids
            .iter()
            .copied()
            .find(|id| gametdb_region(Some(id)) == region)
        {
            return Some(id);
        }
    }
    ids.into_iter().next()
}

fn gametdb_artwork(system: SystemId) -> Option<(&'static str, &'static str)> {
    match system {
        SystemId::GameCube | SystemId::Wii => Some(("wii", "png")),
        SystemId::Nds => Some(("ds", "jpg")),
        SystemId::WiiU => Some(("wiiu", "jpg")),
        _ => None,
    }
}

fn gametdb_cover_url(
    work: &Value,
    system: SystemId,
    release: Option<&Value>,
    serial: Option<&str>,
) -> Option<String> {
    let (directory, extension) = gametdb_artwork(system)?;
    let id = gametdb_id(work, system, release, serial)?;
    if !id.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return None;
    }
    Some(format!(
        "https://art.gametdb.com/{directory}/cover/{}/{}.{extension}",
        release_region(release).unwrap_or_else(|| gametdb_region(Some(id))),
        id.to_ascii_uppercase(),
    ))
}

fn selected_release<'a>(
    work: &'a Value,
    system: SystemId,
    serial: Option<&str>,
) -> Option<&'a Value> {
    let releases = work.get("releases")?.as_array()?;
    let on_platform = |release: &&Value| {
        release.get("platform_id").and_then(Value::as_str) == Some(platform(system))
    };
    if let Some(serial) = serial {
        let code = serial.get(..4).unwrap_or(serial);
        if let Some(release) = releases.iter().filter(on_platform).find(|release| {
            release
                .get("serial")
                .and_then(Value::as_str)
                .is_some_and(|value| {
                    value
                        .to_ascii_uppercase()
                        .contains(&code.to_ascii_uppercase())
                })
        }) {
            return Some(release);
        }
    }
    releases.iter().find(on_platform)
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn metadata(
    work: &Value,
    system: SystemId,
    release: Option<&Value>,
    serial: Option<&str>,
    confidence: u8,
) -> Option<GameMetadata> {
    let title = title_from_work(work)?.to_owned();
    let alternate_titles = work
        .get("titles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| text(entry.get("title")))
        .filter(|other| normalise(other) != normalise(&title))
        .collect();
    let date = release
        .and_then(|item| {
            let year = item.get("year")?.as_str()?;
            let month = item.get("month").and_then(Value::as_u64);
            let day = item.get("day").and_then(Value::as_u64);
            Some(match (month, day) {
                (Some(month), Some(day)) => format!("{year}-{month:02}-{day:02}"),
                (Some(month), None) => format!("{year}-{month:02}"),
                _ => year.to_owned(),
            })
        })
        .or_else(|| {
            work.get("first_year")
                .and_then(Value::as_u64)
                .map(|y| y.to_string())
        });
    let genre = release
        .and_then(|item| item.get("genre_raw"))
        .and_then(Value::as_str)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let cover_url = gametdb_cover_url(work, system, release, serial);
    Some(GameMetadata {
        canonical_title: title,
        alternate_titles,
        developer: release.and_then(|item| text(item.get("developer_raw"))),
        publisher: release.and_then(|item| text(item.get("publisher_raw"))),
        release_date: date,
        genres: genre,
        players: release
            .and_then(|item| item.get("players_max"))
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok()),
        region: release.and_then(|item| text(item.get("region"))),
        cover_source: cover_url.as_ref().map(|_| "GameTDB".into()),
        cover_url,
        source: "Title Screen".into(),
        source_id: work
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        confidence,
        ..Default::default()
    })
}

fn work(id: &str) -> Result<Value, String> {
    get(&format!("{BASE}/works/{}", encode(id)))
}

const MAX_COVER_BYTES: u64 = 10 * 1024 * 1024;

/// Downloads artwork only from the expected GameTDB HTTPS host. A complete PNG
/// is first written beside the destination, then renamed so scans never observe
/// a partial image.
pub fn artwork_extension(url: &str) -> Option<&'static str> {
    let path = url.split(['?', '#']).next()?;
    if path.ends_with(".png") {
        Some("png")
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        Some("jpg")
    } else {
        None
    }
}

fn valid_artwork_bytes(format: &str, bytes: &[u8]) -> bool {
    match format {
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpg" => bytes.starts_with(b"\xff\xd8\xff"),
        _ => false,
    }
}

pub fn cache_cover(url: &str, destination: &Path) -> Result<PathBuf, String> {
    if !url.starts_with("https://art.gametdb.com/") {
        return Err("unsupported artwork host".into());
    }
    let expected_format = artwork_extension(url).ok_or("unsupported artwork format")?;
    if destination.extension().and_then(|value| value.to_str()) != Some(expected_format) {
        return Err("artwork destination extension does not match its format".into());
    }
    if destination.is_file() {
        let existing = std::fs::read(destination).map_err(|error| error.to_string())?;
        if existing.len() as u64 <= MAX_COVER_BYTES
            && valid_artwork_bytes(expected_format, &existing)
        {
            return Ok(destination
                .canonicalize()
                .unwrap_or_else(|_| destination.to_path_buf()));
        }
        std::fs::remove_file(destination).map_err(|error| error.to_string())?;
    }
    let mut response = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|error| error.to_string())?;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let content_type_matches = match expected_format {
        "png" => content_type.starts_with("image/png"),
        "jpg" => content_type.starts_with("image/jpeg"),
        _ => false,
    };
    if !content_type_matches {
        return Err(format!("unexpected artwork content type: {content_type}"));
    }
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_COVER_BYTES)
        .read_to_vec()
        .map_err(|error| error.to_string())?;
    if !valid_artwork_bytes(expected_format, &bytes) {
        return Err(format!("artwork is not a valid {expected_format} file"));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| "artwork cache has no parent directory".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = destination.with_extension(format!("{expected_format}.part"));
    std::fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    std::fs::rename(&temporary, destination).map_err(|error| error.to_string())?;
    Ok(destination
        .canonicalize()
        .unwrap_or_else(|_| destination.to_path_buf()))
}

/// Looks up one game. Exact checksum/serial matches win; title search is only
/// accepted as lower-confidence descriptive metadata and must never authorise
/// a binary patch by itself.
pub fn lookup_title_screen(game: &Game) -> Result<Option<GameMetadata>, String> {
    let exact_url = if let (Some(crc), Some(size)) = (&game.identity.crc32, game.identity.file_size)
    {
        Some(format!("{BASE}/lookup?crc32={}&size={size}", encode(crc)))
    } else {
        game.identity
            .serial
            .as_ref()
            .map(|serial| format!("{BASE}/lookup?serial={}", encode(serial)))
    };
    if let Some(url) = exact_url {
        let answer = get(&url)?;
        if let Some(found) = answer
            .get("matches")
            .and_then(Value::as_array)
            .and_then(|matches| {
                matches.iter().find(|item| {
                    item.get("dump")
                        .and_then(|dump| dump.get("platform"))
                        .and_then(Value::as_str)
                        == Some(platform(game.system))
                })
            })
        {
            if let Some(id) = found
                .get("work")
                .and_then(|w| w.get("id"))
                .and_then(Value::as_str)
            {
                let details = work(id)?;
                return Ok(metadata(
                    &details,
                    game.system,
                    selected_release(&details, game.system, game.identity.serial.as_deref()),
                    game.identity.serial.as_deref(),
                    100,
                ));
            }
        }
    }

    let search = get(&format!(
        "{BASE}/search?q={}&platform={}",
        encode(&game.title),
        platform(game.system)
    ))?;
    let candidates = search.get("results").and_then(Value::as_array);
    for candidate in candidates.into_iter().flatten().take(5) {
        let Some(id) = candidate.get("id").and_then(Value::as_str) else {
            continue;
        };
        let candidate_title = candidate
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if normalise(candidate_title) != normalise(&game.title) {
            continue;
        }
        let details = work(id)?;
        let exact_link = game
            .identity
            .serial
            .as_deref()
            .is_some_and(|serial| link_matches(&details, serial));
        return Ok(metadata(
            &details,
            game.system,
            selected_release(&details, game.system, game.identity.serial.as_deref()),
            game.identity.serial.as_deref(),
            if exact_link { 100 } else { 60 },
        ));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_queries_and_normalises_titles() {
        assert_eq!(encode("Mario & Luigi"), "Mario%20%26%20Luigi");
        assert_eq!(normalise("Super Mario: Sunshine!"), "supermariosunshine");
    }

    #[test]
    fn builds_safe_gametdb_cover_urls() {
        let work = serde_json::json!({
            "links": [{"kind": "gametdb", "value": "GMSE01"}]
        });
        assert_eq!(
            gametdb_cover_url(&work, SystemId::GameCube, None, Some("GMSE01")).as_deref(),
            Some("https://art.gametdb.com/wii/cover/US/GMSE01.png")
        );
        assert_eq!(
            gametdb_cover_url(&work, SystemId::GameCube, None, Some("GMSP01")).as_deref(),
            Some("https://art.gametdb.com/wii/cover/US/GMSE01.png")
        );
        assert_eq!(
            gametdb_cover_url(&work, SystemId::GameCube, None, None).as_deref(),
            Some("https://art.gametdb.com/wii/cover/US/GMSE01.png")
        );
    }

    #[test]
    fn matches_gamecube_box_serial_to_regional_gametdb_id() {
        let work = serde_json::json!({
            "links": [
                {"kind": "gametdb", "value": "GMSE01"},
                {"kind": "gametdb", "value": "GMSJ01"},
                {"kind": "gametdb", "value": "GMSP01"}
            ],
            "releases": [{
                "platform_id": "gc",
                "region": "AU",
                "serial": "DL-DOL-GMSP-EUR;DL-DOL-GMSP-EUR-01"
            }]
        });
        let release = selected_release(&work, SystemId::GameCube, None);
        assert_eq!(
            gametdb_cover_url(&work, SystemId::GameCube, release, None).as_deref(),
            Some("https://art.gametdb.com/wii/cover/AU/GMSP01.png")
        );
    }

    #[test]
    fn selects_matching_ds_edition_and_jpeg_format() {
        let work = serde_json::json!({
            "links": [
                {"kind": "gametdb", "value": "A39E"},
                {"kind": "gametdb", "value": "AMCJ"},
                {"kind": "gametdb", "value": "AMCE"}
            ],
            "releases": [{
                "platform_id": "nds",
                "region": "US",
                "serial": "NTR-AMCE-USA"
            }]
        });
        let release = selected_release(&work, SystemId::Nds, Some("AMCE"));
        assert_eq!(
            gametdb_cover_url(&work, SystemId::Nds, release, Some("AMCE")).as_deref(),
            Some("https://art.gametdb.com/ds/cover/US/AMCE.jpg")
        );
    }

    #[test]
    fn builds_wiiu_jpeg_and_rejects_unsupported_system_artwork() {
        let work = serde_json::json!({
            "links": [{"kind": "gametdb", "value": "ARDE01"}]
        });
        assert_eq!(
            gametdb_cover_url(&work, SystemId::WiiU, None, Some("ARDE")).as_deref(),
            Some("https://art.gametdb.com/wiiu/cover/US/ARDE01.jpg")
        );
        assert!(link_matches(&work, "ARDE"));
        assert_eq!(
            gametdb_cover_url(&work, SystemId::Ps2, None, Some("ARDE01")),
            None
        );
    }

    #[test]
    fn recognises_supported_artwork_extensions() {
        assert_eq!(artwork_extension("https://example.test/a.png"), Some("png"));
        assert_eq!(
            artwork_extension("https://example.test/a.jpg?revision=1"),
            Some("jpg")
        );
        assert_eq!(artwork_extension("https://example.test/a.gif"), None);
        assert!(valid_artwork_bytes("png", b"\x89PNG\r\n\x1a\nrest"));
        assert!(valid_artwork_bytes("jpg", b"\xff\xd8\xffrest"));
        assert!(!valid_artwork_bytes("jpg", b"\x89PNG\r\n\x1a\n"));
    }
}
