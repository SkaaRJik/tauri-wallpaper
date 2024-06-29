// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate sxd_document;

use std::{
    fs,
    fs::File,
    usize,
};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use futures_util::stream::StreamExt;
use rand::Rng;
use reqwest;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::StatusCode;
use tauri::{Manager};
use tokio::runtime::Runtime;

#[tauri::command]
fn my_custom_command() {
    println!("I was invoked from JS!");
}

async fn get_document_html(client: &reqwest::Client, url: String) -> Result<scraper::Html, Box<dyn Error>> {
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .header(ACCEPT, "text/html; charset=utf-8")
        .send()
        .await
        .unwrap();

    let raw_html = match response.status() {
        StatusCode::OK => response.text().await.unwrap(),
        _ => panic!("Something went wrong"),
    };

    let document = scraper::Html::parse_document(&raw_html);
    return Ok(document);
}

async fn download_chunk(href: &str, image_path: PathBuf) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let response = client.get(href).send().await?;

    let mut out = File::create(image_path)?;
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        out.write_all(&chunk)?;
    }

    println!("Загрузка завершена");
    Ok("Good".to_string())
}
async fn download_image(
    pic_index_page: usize,
    pic_page: usize,
    pic_index: usize,
    resolution: String,
    download_folder: String,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let page_url = format!(
        "https://qqq.com/{resolution}/sort/views/page/{pic_page}",
        resolution = resolution,
        pic_page = pic_page
    );

    let document = get_document_html(&client, page_url.clone()).await.unwrap();

    let pictures_selector = scraper::Selector::parse(format!("div.wall-resp:nth-child({})", pic_index_page).as_str()).unwrap();

    if let Some(pic_page_element) = document.select(&pictures_selector).next() {
        let pic_url = pic_page_element.value().attr("href").unwrap();

        let document = get_document_html(&client, pic_url.to_string()).await.unwrap();

        let header_selector = scraper::Selector::parse("div.page-header > h1").unwrap();

        if let Some(header_element) = document.select(&header_selector).next() {
            let header_text = header_element
                .text()
                .collect::<Vec<_>>()
                .concat()
                .replace("\n", "");
            let ext = &pic_url[pic_url.len() - 3..];
            let image_name = format!("{}.{}", header_text, ext);
            let folder_path = std::path::Path::new(&download_folder);

            if !folder_path.exists() {
                fs::create_dir_all(folder_path.clone())?;
            }
            let image_path = folder_path.join(image_name.clone());

            download_chunk(pic_url, image_path);

            return Ok(format!("Image {} was saved", image_name));
        }
    }

    return Err(format!("{page_url} Page not found").into());
}

async fn download_pictures(
    num_of_pictures: u32,
    resolution: String,
    download_folder: String,
) -> Result<String, Box<dyn Error>> {

    for _i in 0..num_of_pictures {
        match download_image(
            1,
            1,
            2,
            resolution.clone(),
            download_folder.clone(),
        ).await {
            Ok(mes) => {
                println!("{}", mes);
            }
            Err(err) => {
                println!("{}", err.to_string())
            }
        }
    }

    Ok("Images downloaded".to_string())
}

fn run_check_and_download() {
    let rt = Runtime::new().unwrap();
    let future = download_pictures(10, "2560x1440".to_string(), "img".to_string());
    let _r = rt.block_on(future);
}

fn setup<'a>(app: &'a mut tauri::App) -> Result<(), Box<dyn Error>> {

    tauri::async_runtime::spawn(async move {
        // also added move here
        match download_pictures(10, "2560x1440".to_string(), "img".to_string()).await {
            Ok(mes) => {
                println!("{}", mes);
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    });
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("Box<dyn Error> while running tauri application");

    println!("main loop");
}
