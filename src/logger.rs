use chrono::Utc;
use std::io::Write;
use anyhow::Result;
use std::path::Path;

/// Simple file-based logger for trading activities
#[derive(Debug)]
pub struct SheetLogger {
    log_file: String,
}

impl SheetLogger {
    pub async fn new(_credentials_path: &str, sheet_id: &str) -> Result<Self> {
        let log_file = format!("logs/trades_{}.csv", sheet_id);

        if let Some(parent) = Path::new(&log_file).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file_exists = std::path::Path::new(&log_file).exists();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        if !file_exists {
            writeln!(file, "timestamp,market_id,side,price,size,status")?;
        }

        Ok(SheetLogger { log_file })
    }

    pub async fn append_row(&self, row_data: Vec<String>) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;

        let row_str = row_data.join(",");
        writeln!(file, "{}", row
