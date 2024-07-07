use std::sync::Arc;
use chrono::{DateTime, Utc};
use log::trace;
use serde_json::Value;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct ImgInfo {
    title: Arc<String>,
    page_url: Arc<String>,
    download_url: Arc<String>,
    download_date: Arc<DateTime<Utc>>,
}
#[derive(Debug, Clone)]
pub struct CvkClientService {
    window: tauri::Window,
    downloaded_images: Arc<Mutex<Vec<ImgInfo>>>,
    seed: 
}


