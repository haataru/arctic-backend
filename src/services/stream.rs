use actix_web::{HttpRequest, HttpResponse, http::header};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn stream_file(req: &HttpRequest, filepath: &str) -> HttpResponse {
    let path = Path::new(filepath);
    
    if !path.exists() {
        return HttpResponse::NotFound().body("File not found");
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open file"),
    };

    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get file metadata"),
    };

    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    if let Some(range_header) = req.headers().get(header::RANGE) {
        if let Ok(range_str) = range_header.to_str() {
            if let Some(range) = parse_range_header(range_str, file_size) {
                return serve_range(file, file_size, range, &mime_type);
            }
        }
    }

    serve_entire_file(file, file_size, &mime_type)
}

fn parse_range_header(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    if !range_str.starts_with("bytes=") {
        return None;
    }

    let range_part = &range_str[6..];
    let parts: Vec<&str> = range_part.split('-').collect();
    
    if parts.len() != 2 {
        return None;
    }

    let start = if parts[0].is_empty() {
        0
    } else {
        match parts[0].parse::<u64>() {
            Ok(s) => s,
            Err(_) => return None,
        }
    };

    let end = if parts[1].is_empty() {
        file_size - 1
    } else {
        match parts[1].parse::<u64>() {
            Ok(e) => e,
            Err(_) => return None,
        }
    };

    if start >= file_size || end >= file_size || start > end {
        return None;
    }

    Some((start, end))
}

fn serve_range(mut file: File, file_size: u64, range: (u64, u64), mime_type: &str) -> HttpResponse {
    let (start, end) = range;
    let content_length = end - start + 1;

    file.seek(SeekFrom::Start(start)).unwrap();
    let mut buffer = vec![0u8; content_length as usize];
    file.read_exact(&mut buffer).unwrap();

    HttpResponse::PartialContent()
        .insert_header((header::CONTENT_TYPE, mime_type.to_string()))
        .insert_header((header::CONTENT_LENGTH, content_length.to_string()))
        .insert_header((header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size)))
        .body(buffer)
}

fn serve_entire_file(mut file: File, file_size: u64, mime_type: &str) -> HttpResponse {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, mime_type.to_string()))
        .insert_header((header::CONTENT_LENGTH, file_size.to_string()))
        .body(buffer)
}
