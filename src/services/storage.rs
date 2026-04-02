use std::fs;
use std::io;
use std::path::Path;
use uuid::Uuid;

pub struct StorageService {
    upload_dir: String,
}

impl StorageService {
    pub fn new(upload_dir: &str) -> Self {
        fs::create_dir_all(upload_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create upload directory: {}", e);
        });
        Self {
            upload_dir: upload_dir.to_string(),
        }
    }

    pub fn save_file(&self, file_data: &[u8], extension: &str) -> io::Result<String> {
        let filename = format!("{}.{}", Uuid::new_v4(), extension);
        let filepath = format!("{}/{}", self.upload_dir, filename);
        fs::write(&filepath, file_data)?;
        Ok(filepath)
    }

    pub fn delete_file(&self, filepath: &str) -> io::Result<()> {
        if Path::new(filepath).exists() {
            fs::remove_file(filepath)?;
        }
        Ok(())
    }
}
