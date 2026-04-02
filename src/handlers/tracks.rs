use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use sqlx::SqlitePool;

use crate::models::{Track, TrackResponse, UpdateTrackRequest};
use crate::services::{StorageService, stream_file};

pub struct AppState {
    pub pool: SqlitePool,
    pub storage: StorageService,
    pub base_url: String,
}

pub async fn get_tracks(state: web::Data<AppState>) -> Result<HttpResponse> {
    let tracks = sqlx::query_as::<_, Track>(
        "SELECT id, title, artist, file_path, cover_path, created_at FROM tracks"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let response: Vec<TrackResponse> = tracks.iter().map(|t| t.to_response(&state.base_url)).collect();
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_track(path: web::Path<i64>, state: web::Data<AppState>) -> Result<HttpResponse> {
    let track_id = path.into_inner();
    let track = sqlx::query_as::<_, Track>(
        "SELECT id, title, artist, file_path, cover_path, created_at FROM tracks WHERE id = ?"
    )
    .bind(track_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Track not found"))?;
    
    Ok(HttpResponse::Ok().json(track.to_response(&state.base_url)))
}

pub async fn create_track(mut payload: Multipart, state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut title = String::new();
    let mut artist = String::new();
    let mut audio_data: Option<Vec<u8>> = None;
    let mut audio_ext = String::from("mp3");
    let mut cover_data: Option<Vec<u8>> = None;
    let mut cover_ext = String::from("jpg");
    
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
        let content_disposition = field.content_disposition().clone();
        let field_name = content_disposition.get_name().unwrap_or("");
        
        match field_name {
            "title" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                    title.push_str(&String::from_utf8_lossy(&data));
                }
            }
            "artist" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                    artist.push_str(&String::from_utf8_lossy(&data));
                }
            }
            "audio" => {
                let filename = content_disposition.get_filename().unwrap_or("audio.mp3");
                audio_ext = filename.split('.').last().unwrap_or("mp3").to_string();
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk_data = chunk.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                    data.extend_from_slice(&chunk_data);
                }
                audio_data = Some(data);
            }
            "cover" => {
                let filename = content_disposition.get_filename().unwrap_or("cover.jpg");
                cover_ext = filename.split('.').last().unwrap_or("jpg").to_string();
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk_data = chunk.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
                    data.extend_from_slice(&chunk_data);
                }
                cover_data = Some(data);
            }
            _ => {}
        }
    }
    
    if title.is_empty() || artist.is_empty() {
        return Err(actix_web::error::ErrorBadRequest("Title and artist are required"));
    }
    
    let audio_data = audio_data.ok_or_else(|| actix_web::error::ErrorBadRequest("Audio file is required"))?;
    let audio_path = state.storage.save_file(&audio_data, &audio_ext)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let cover_path = if let Some(cover_data) = cover_data {
        Some(state.storage.save_file(&cover_data, &cover_ext)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?)
    } else {
        None
    };
    
    let result = sqlx::query(
        "INSERT INTO tracks (title, artist, file_path, cover_path) VALUES (?, ?, ?, ?)"
    )
    .bind(title)
    .bind(artist)
    .bind(audio_path)
    .bind(cover_path)
    .execute(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let track_id = result.last_insert_rowid();
    let track = sqlx::query_as::<_, Track>(
        "SELECT id, title, artist, file_path, cover_path, created_at FROM tracks WHERE id = ?"
    )
    .bind(track_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(track.to_response(&state.base_url)))
}

pub async fn update_track(
    path: web::Path<i64>,
    body: web::Json<UpdateTrackRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let track_id = path.into_inner();
    
    let _existing_track = sqlx::query_as::<_, Track>(
        "SELECT id, title, artist, file_path, cover_path, created_at FROM tracks WHERE id = ?"
    )
    .bind(track_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Track not found"))?;
    
    if let Some(title) = &body.title {
        sqlx::query("UPDATE tracks SET title = ? WHERE id = ?")
            .bind(title)
            .bind(track_id)
            .execute(&state.pool)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }
    
    if let Some(artist) = &body.artist {
        sqlx::query("UPDATE tracks SET artist = ? WHERE id = ?")
            .bind(artist)
            .bind(track_id)
            .execute(&state.pool)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }
    
    let track = sqlx::query_as::<_, Track>(
        "SELECT id, title, artist, file_path, cover_path, created_at FROM tracks WHERE id = ?"
    )
    .bind(track_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(track.to_response(&state.base_url)))
}

pub async fn delete_track(path: web::Path<i64>, state: web::Data<AppState>) -> Result<HttpResponse> {
    let track_id = path.into_inner();
    let track = sqlx::query_as::<_, Track>(
        "SELECT id, title, artist, file_path, cover_path, created_at FROM tracks WHERE id = ?"
    )
    .bind(track_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Track not found"))?;
    
    state.storage.delete_file(&track.file_path)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if let Some(cover_path) = &track.cover_path {
        state.storage.delete_file(cover_path)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }
    
    sqlx::query("DELETE FROM tracks WHERE id = ?")
        .bind(track_id)
        .execute(&state.pool)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

pub async fn stream_track(req: HttpRequest, path: web::Path<i64>, state: web::Data<AppState>) -> Result<HttpResponse> {
    let track_id = path.into_inner();
    let file_path: String = sqlx::query_scalar(
        "SELECT file_path FROM tracks WHERE id = ?"
    )
    .bind(track_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Track not found"))?;
    
    Ok(stream_file(&req, &file_path))
}

pub async fn get_cover(path: web::Path<i64>, state: web::Data<AppState>) -> Result<HttpResponse> {
    let track_id = path.into_inner();
    let cover_path: Option<String> = sqlx::query_scalar(
        "SELECT cover_path FROM tracks WHERE id = ?"
    )
    .bind(track_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Track not found"))?;
    
    match cover_path {
        Some(path) => {
            let file = std::fs::read(&path)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            let mime_type = mime_guess::from_path(&path).first_or_octet_stream().to_string();
            Ok(HttpResponse::Ok().insert_header(("Content-Type", mime_type)).body(file))
        }
        None => Ok(HttpResponse::Ok()
            .insert_header(("Content-Type", "image/svg+xml"))
            .body("<svg xmlns='http://www.w3.org/2000/svg' width='100' height='100' viewBox='0 0 100 100'><rect width='100' height='100' fill='%23333'/><text x='50' y='50' font-family='Arial' font-size='12' fill='%23fff' text-anchor='middle' dy='.3em'>No Cover</text></svg>"))
    }
}
