use std::{io::ErrorKind, path::Path};

use rocket::http::{ContentType, Header};
use tokio::fs::File;

use crate::models::files::FetchResponse;

struct StaticFile<'a> {
    file: File,
    path: &'a Path,
    content_type: Option<ContentType>,
}

#[get("/<name>", rank = 1)]
pub async fn get_static_file<'a>(name: String) -> Result<FetchResponse<'a>, String> {
    let StaticFile {
        file,
        path,
        content_type,
    } = get_file(&name).await.map_err(|e| e.to_string())?;

    Ok(FetchResponse {
        file,
        disposition: Header::new(
            "Content-Disposition",
            format!(
                "inline; filename=\"{}\"",
                path.file_name().unwrap().to_str().unwrap()
            ),
        ),
        content_type: content_type.unwrap_or(ContentType::Any),
    })
}

#[get("/<name>/download", rank = 1)]
pub async fn download_static_file<'a>(name: String) -> Result<FetchResponse<'a>, String> {
    let StaticFile {
        file,
        path,
        content_type,
    } = get_file(&name).await.map_err(|e| e.to_string())?;

    Ok(FetchResponse {
        file,
        disposition: Header::new(
            "Content-Disposition",
            format!(
                "attachment; filename=\"{}\"",
                path.file_name().unwrap().to_str().unwrap()
            ),
        ),
        content_type: content_type.unwrap_or(ContentType::Any),
    })
}

async fn get_file(name: &str) -> Result<StaticFile, String> {
    let path = Path::new(name)
        .file_name()
        .map(Path::new)
        .ok_or_else(|| "Invalid file name".to_string())?;

    let extension = path.extension();
    let content_type = match extension {
        Some(extension) => ContentType::from_extension(
            extension
                .to_str()
                .ok_or_else(|| "Invalid file extension".to_string())?,
        ),
        None => None,
    };

    let file = File::open(Path::new("files/static").join(path))
        .await
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                "File not found".to_string()
            } else {
                "Failed to get static file from storage".to_string()
            }
        })?;

    log::debug!("Fetched static file {}", name);

    Ok(StaticFile {
        file,
        path,
        content_type,
    })
}
