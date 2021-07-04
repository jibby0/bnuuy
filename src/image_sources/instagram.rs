use std::iter::zip;
use std::collections::HashMap;
use url::Url;
use std::io::Read;
use std::fs::DirEntry;
use crate::{db, image_sources::ImageSource, Result};
use std::fs::File;

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instagram {
    username: String,
    password: String,
    image_folder: String,
    instagram_pages: Vec<String>,
}

const MAXIMUM_IMAGES_TO_RETAIN: u32 = 100;

impl ImageSource for Instagram {
    fn update_image_paths(&self, pool: &mut db::Pool) -> Result<()> {
        log::debug!("Adding new images & wiping old images..");

        let pages = self.instagram_pages.join(",");
        let output = Command::new("instagram-scraper")
            .arg(pages)
            .arg("--destination")
            .arg(&self.image_folder)
            .arg("--retain-username")
            .arg("--media-metadata")
            .arg("--media-types")
            .arg("image")
            .arg("--latest")
            .arg("--login-user")
            .arg(&self.username)
            .arg("--login-pass")
            .arg(&self.password)
            .arg("--maximum")
            .arg(format!("{}", MAXIMUM_IMAGES_TO_RETAIN))
            .output()?;

        let stdout_utf8 = std::str::from_utf8(output.stdout.as_slice())?;
        if !output.status.success() {
            return Err(format!(
                "instagram-scraper executed with failing error code {}: {}",
                output.status, stdout_utf8
            )
            .into());
        }

        if stdout_utf8.contains("Login failed for") {
            return Err(format!("instagram-scraper failed login: {}", stdout_utf8).into());
        }

        let conn = db::DbConn(pool.get()?);

        let mut images = Vec::new();

        // TODO this should map to a vec of results or something, this is awful
        //  From the base folder, flat_map to a subfolder & its metadata
        //  From the subfolder & metadata, flat_map to images that pass the accessibilty check
        let image_paths = fs::read_dir(&self.image_folder)?.filter_map(
            |path| match path {
                    Ok(r) => Some(r),
                    Err(err) => {
                        log::warn!("{}", err);
                        None
                    }
            }
        );

        if self.using_metadata() {
            let mut metadata: HashMap<String, String> = HashMap::new();
            for p in image_paths {
                match Instagram::get_metadata(p) {
                    Ok(r) => {
                        for (k, v) in r {
                            metadata.insert(k, v);
                        }
                    }
                    Err(err) => log::warn!("{}", err)
                };
            }
            // TODO some regex
        }

        for entry in WalkDir::new(&self.image_folder) {
            let e = match entry {
                Err(err) => {
                    log::warn!("{}", err);
                    continue;
                }
                Ok(e) => e,
            };
            if e.file_type().is_file() {
                let path = e.path().to_str().unwrap().to_string();
                if path.ends_with("jpg") || path.ends_with("jpeg") {
                    let metadata = fs::metadata(path.clone()).unwrap();
                    let timestamp = metadata
                        .created()
                        .unwrap_or_else(|_| metadata.modified().unwrap());
                    images.push((path, timestamp));
                }
            }
        }

        // Reverse sort by date
        images.sort_by(|(_, time_a), (_, time_b)| time_b.partial_cmp(time_a).unwrap());

        // Retain the N most recent
        images.truncate(MAXIMUM_IMAGES_TO_RETAIN as usize);

        let dogs = images
            .iter()
            .map(|(path, _time)| db::dogs::Dog { path: path.clone() })
            .collect();

        // TODO filter using metadata & allowed accesssibility options
        db::dogs::delete_all(&conn)?;
        db::dogs::insert_many(dogs, &conn)?;

        log::debug!("Done adding new paths");
        Ok(())
    }

}

impl Instagram {
    fn get_metadata(entry: DirEntry) -> Result<HashMap<String, String>> {
        let metadata_json = Instagram::find_metadata_file(entry)?;
        return Instagram::parse_metadata(metadata_json)
    }

    fn find_metadata_file(entry: std::fs::DirEntry) -> Result<serde_json::Value> {
        let path = entry.path();

        if !path.is_dir() {
            return Err("Not a directory".into())
        }
        let metadata_file_name = path.file_name().ok_or(
            "Can't get file name"
        )?.to_str().ok_or(
            "Can't get file name as str"
        )?;

        let mut metadata_fp = File::open(path.join(metadata_file_name))?;
        let mut metadata_str = String::new();
        metadata_fp.read_to_string(&mut metadata_str)?;
        let metadata = serde_json::from_str(&metadata_str)?;

        log::debug!("Pulled metadata for {:?}", path);
        return Ok(metadata);
    }

    fn parse_metadata(metadata_json: serde_json::Value) -> Result<HashMap<String, String>> {
        let entries = match metadata_json.get("GraphImages") {
            Some(serde_json::Value::Array(p)) => p,
            _ => return Err("Missing or invalid GraphImages in metadata".into()),
        };
        let mut images_to_accessibility: HashMap<String, String> = HashMap::new();

        for e in entries {
            match Instagram::parse_graph_image_entry(e) {
                Some(map) => {
                    for (k, v) in map {
                        images_to_accessibility.insert(k, v);
                    }
                },
                None => (),
            }
        }
        Ok(images_to_accessibility)
    }

    fn parse_graph_image_entry(entry: &serde_json::Value) -> Option<HashMap<String, String>> {
        let urls = match entry.get("urls") {
            Some(serde_json::Value::Array(u)) => u,
            _ => return None,
        };
        let accessibility_captions = match entry.get("accessibility_captions") {
            Some(serde_json::Value::Array(a)) => a,
            _ => return None,
        };

        let mut images_to_accessibility: HashMap<String, String> = HashMap::new();
        for (cap, url) in zip(urls.iter(), accessibility_captions.iter()) {
            let (caption, url) = match (cap, url) {
                (serde_json::Value::String(c), serde_json::Value::String(u)) => (c, u),
                _ => continue,
            };

            let parsed_url = match Url::parse(&url) {
                Ok(u) => u,
                Err(_) => continue,
            };
            let image = match parsed_url.path_segments().and_then(|s| s.last()) {
                Some(last) => last,
                None => continue,
            };
            images_to_accessibility.insert(image.to_string(), caption.to_string());
        }
        Some(images_to_accessibility)

    }

    fn using_metadata(&self) -> bool {
        // TODO some check for allow/block regex
        true
    }
}