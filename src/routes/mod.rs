use rocket::{routes, Route};

mod index;
mod static_files;

pub fn routes() -> Vec<Route> {
    routes![
        index::upload_attachment,
        index::get_attachment,
        index::download_attachment,
        static_files::get_static_file,
        static_files::download_static_file,
    ]
}
