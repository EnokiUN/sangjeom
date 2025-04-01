use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use rocket::{
    fs::TempFile,
    http::{ContentType, Header},
    FromForm, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, Sqlite};
use tokio::fs;

use crate::id::IdGen;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileData {
    pub id: i64,
    pub name: String,
    pub owner: String,
}

pub struct File {
    pub id: i64,
    pub file_id: i64,
    pub name: String,
    pub content_type: String,
    pub hash: String,
    pub owner: String,
}

#[derive(Debug, Responder)]
pub struct FetchResponse<'a> {
    pub file: fs::File,
    pub disposition: Header<'a>,
    pub content_type: ContentType,
}

#[derive(Debug, FromForm)]
pub struct FileUpload<'a> {
    pub file: TempFile<'a>,
}

impl File {
    pub async fn create<'a>(
        mut file: TempFile<'a>,
        id_generator: &mut IdGen,
        owner: String,
        db: &mut PoolConnection<Sqlite>,
    ) -> Result<FileData> {
        if file.len() == 0 {
            bail!("Cannot upload an empty file");
        }

        let id = id_generator.generate();
        let path = PathBuf::from(format!("files/{}", id));
        let name = match file.raw_name() {
            Some(name) => PathBuf::from(name.dangerous_unsafe_unsanitized_raw().as_str())
                .file_name()
                .map(|n| n.to_str().unwrap_or("attachment"))
                .unwrap_or("attachment")
                .to_string(),
            None => "attachment".to_string(),
        };
        if name.is_empty() || name.len() > 256 {
            bail!("Invalid file name. File name must be between 1 and 256 characters long");
        }
        file.persist_to(&path).await.unwrap();
        let data = fs::read(&path).await.unwrap();

        let hash = sha256::digest(&data[..]);
        let file = if let Ok((file_id, content_type, owner)) = sqlx::query!(
            "
SELECT file_id, content_type, owner
FROM files
WHERE hash = $1
            ",
            hash,
        )
        .fetch_one(&mut **db)
        .await
        .map(|f| (f.file_id, f.content_type, f.owner))
        {
            fs::remove_file(path).await.unwrap();
            sqlx::query!(
                "
INSERT INTO files(id, file_id, name, content_type, hash, owner)
VALUES($1, $2, $3, $4, $5, $6)
                ",
                id,
                file_id,
                name,
                content_type,
                hash,
                owner,
            )
            .execute(&mut **db)
            .await
            .unwrap();

            Self {
                id,
                file_id,
                name,
                content_type,
                hash,
                owner,
            }
        } else {
            let file = tokio::task::spawn_blocking(move || {
                let mut mime = tree_magic::from_u8(&data);
                if mime == "application/x-riff" && name.ends_with(".webp") {
                    // tree magic bug
                    mime = "image/webp".to_string();
                }
                Ok::<Self, anyhow::Error>(Self {
                    id,
                    file_id: id,
                    name,
                    content_type: mime,
                    hash,
                    owner,
                })
            })
            .await
            .unwrap()?;
            sqlx::query!(
                "
INSERT INTO files(id, file_id, name, content_type, hash, owner)
VALUES($1, $2, $3, $4, $5, $6)
                ",
                file.id,
                file.id,
                file.name,
                file.content_type,
                file.hash,
                file.owner,
            )
            .execute(&mut **db)
            .await
            .unwrap();

            file
        };

        Ok(file.get_file_data())
    }

    pub async fn get<'a>(id: i64, db: &mut PoolConnection<Sqlite>) -> Option<Self> {
        sqlx::query_as!(
            Self,
            "
SELECT *
FROM files
WHERE id = $1
            ",
            id,
        )
        .fetch_one(&mut **db)
        .await
        .ok()
    }

    pub async fn fetch_file<'a>(
        id: i64,
        db: &mut PoolConnection<Sqlite>,
    ) -> Result<FetchResponse<'a>> {
        let file_data = Self::get(id, db)
            .await
            .ok_or_else(|| anyhow!("File not found"))?;
        let file = fs::File::open(format!("files/{}", file_data.file_id))
            .await
            .map_err(|e| {
                log::error!(
                    "Could not fetch file {} with id {}: {:?}",
                    file_data.name,
                    file_data.id,
                    e
                );
                anyhow!("Error fetching file")
            })?;
        Ok(FetchResponse {
            file,
            disposition: Header::new(
                "Content-Disposition",
                format!("inline; filename=\"{}\"", file_data.name),
            ),
            content_type: ContentType::parse_flexible(&file_data.content_type).unwrap(),
        })
    }

    pub async fn fetch_file_download<'a>(
        id: i64,
        db: &mut PoolConnection<Sqlite>,
    ) -> Result<FetchResponse<'a>> {
        let file_data = Self::get(id, db)
            .await
            .ok_or_else(|| anyhow!("File not found"))?;
        let file = fs::File::open(format!("files/{}", file_data.file_id))
            .await
            .map_err(|e| {
                log::error!(
                    "Could not fetch file {} with id {}: {:?}",
                    file_data.name,
                    file_data.id,
                    e
                );
                anyhow!("Error fetching file")
            })?;
        Ok(FetchResponse {
            file,
            disposition: Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", file_data.name),
            ),
            content_type: ContentType::parse_flexible(&file_data.content_type).unwrap(),
        })
    }

    #[allow(dead_code)]
    pub async fn fetch_file_data<'a>(id: i64, db: &mut PoolConnection<Sqlite>) -> Result<FileData> {
        Self::get(id, db)
            .await
            .ok_or_else(|| anyhow!("File not found"))
            .map(|f| f.get_file_data())
    }

    fn get_file_data(self) -> FileData {
        FileData {
            id: self.id,
            name: self.name,
            owner: self.owner,
        }
    }
}
