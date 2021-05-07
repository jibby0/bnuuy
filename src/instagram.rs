use crate::{db, Result, IMAGE_FOLDER, INSTA_PAGES, INSTA_PASSWORD, INSTA_USERNAME};

use std::fs;
use std::process::Command;
use walkdir::WalkDir;
pub fn update_image_paths(pool: &mut db::Pool) -> Result<()> {
    log::debug!("Adding new images & wiping old images..");

    let pages = INSTA_PAGES.join(",");
    let output = Command::new("instagram-scraper")
        .arg(pages)
        .arg("--destination")
        .arg(String::from(IMAGE_FOLDER))
        .arg("--retain-username")
        .arg("--media-metadata")
        .arg("--media-types")
        .arg("image")
        .arg("--latest")
        .arg("--login-user")
        .arg(String::from(INSTA_USERNAME))
        .arg("--login-pass")
        .arg(String::from(INSTA_PASSWORD))
        .arg("--maximum")
        .arg("100")
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
    for entry in WalkDir::new(IMAGE_FOLDER) {
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
                let created = fs::metadata(path.clone()).unwrap().created().unwrap();
                images.push((path, created));
            }
        }
    }

    // Reverse sort by date
    images.sort_by(|(_, time_a), (_, time_b)| time_b.partial_cmp(time_a).unwrap());

    // Retain the N most recent
    images.truncate(1000);

    let dogs = images
        .iter()
        .map(|(path, _time)| db::dogs::Dog { path: path.clone() })
        .collect();

    db::dogs::delete_all(&conn)?;
    db::dogs::insert_many(dogs, &conn)?;

    log::debug!("Done adding new paths");
    Ok(())
}
