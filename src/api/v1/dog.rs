use crate::api::v1::{Resp, RespErr};
use crate::db::{dogs, DbConn};

use rocket::{http::Status, response::content};
use std::io::prelude::Read;

use std::fs;

#[get("/dog")]
pub fn dog(conn: DbConn) -> Resp<content::Html<String>> {
    let path = match dogs::get_random(&conn) {
        Ok(dog) => dog.path,
        Err(_) => return Err(RespErr(Status::NotFound)),
    };
    log::debug!("Pulling image from {}", path);
    let b64_image = image_as_b64(path)?;

    Ok(content::Html(format!(
        "<html><center>
        <img src=\"data:image/jpeg;base64, {}\"/></center>
        <script>setTimeout(function(){{ window.location.reload(1); }}, 10000);
        </script></html>",
        b64_image
    )))
}

fn image_as_b64(path: String) -> Resp<String> {
    let mut f = fs::File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(base64::encode(buffer))
}
