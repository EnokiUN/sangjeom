use rocket::{form::Form, serde::json::Json, State};
use rocket_db_pools::Connection;
use tokio::sync::Mutex;

use crate::{
    auth::TokenAuth,
    id::IdGen,
    models::files::{FetchResponse, File, FileData, FileUpload},
    DB,
};

#[post("/", data = "<upload>")]
pub async fn upload_attachment<'a>(
    upload: Form<FileUpload<'a>>,
    mut db: Connection<DB>,
    gen: &State<Mutex<IdGen>>,
    auth: TokenAuth,
) -> Result<Json<FileData>, String> {
    let upload = upload.into_inner();
    File::create(
        upload.file,
        &mut *gen.inner().lock().await,
        auth.owner,
        &mut db,
    )
    .await
    .map(Json)
    .map_err(|e| e.to_string())
}

#[get("/<id>")]
pub async fn get_attachment<'a>(
    id: i64,
    mut db: Connection<DB>,
) -> Result<FetchResponse<'a>, String> {
    File::fetch_file(id, &mut db)
        .await
        .map_err(|e| e.to_string())
}

#[get("/<id>/download")]
pub async fn download_attachment<'a>(
    id: i64,
    mut db: Connection<DB>,
) -> Result<FetchResponse<'a>, String> {
    File::fetch_file_download(id, &mut db)
        .await
        .map_err(|e| e.to_string())
}
