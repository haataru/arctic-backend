use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;

mod db;
mod handlers;
mod models;
mod services;

use handlers::tracks::AppState;
use services::StorageService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:arctic.db".to_string());
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let pool = db::init_db(&db_path).await.expect("Failed to initialize database");
    let storage = StorageService::new(&upload_dir);
    let app_state = web::Data::new(AppState { pool, storage, base_url });
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(handlers::health::health_check))
                    .route("/tracks", web::get().to(handlers::tracks::get_tracks))
                    .route("/tracks", web::post().to(handlers::tracks::create_track))
                    .route("/tracks/{id}", web::get().to(handlers::tracks::get_track))
                    .route("/tracks/{id}", web::put().to(handlers::tracks::update_track))
                    .route("/tracks/{id}", web::delete().to(handlers::tracks::delete_track))
                    .route("/tracks/{id}/stream", web::get().to(handlers::tracks::stream_track))
                    .route("/tracks/{id}/cover", web::get().to(handlers::tracks::get_cover)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
