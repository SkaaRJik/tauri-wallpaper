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

async fn get_document_html(client: &reqwest::Client, url: String) -> scraper::Html {
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
    return document;
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
    let blacklist_games = [
        "pubg", "playerunknown", "fifa", "nba", "football", "genshin", "fortnite",
    ];

    let client = reqwest::Client::new();

    let page_url = format!(
        "https://hdqwalls.com/category/games-wallpapers/{resolution}/sort/views/page/{pic_page}",
        resolution = resolution,
        pic_page = pic_page
    );

    let document = get_document_html(&client, page_url.clone()).await;

    let pictures_selector = scraper::Selector::parse(format!("div.wall-resp:nth-child({})", pic_index_page).as_str()).unwrap();

    if let Some(pic_page_element) = document.select(&pictures_selector).next() {
        let pic_url = pic_page_element.value().attr("href").unwrap();

        let document = get_document_html(&client, pic_url.to_string()).await;

        let header_selector = scraper::Selector::parse("div.page-header > h1").unwrap();

        if let Some(header_element) = document.select(&header_selector).next() {
            let header_text = header_element
                .text()
                .collect::<Vec<_>>()
                .concat()
                .replace("\n", "");
            let trimmed_header_text = header_text.trim().to_lowercase();
            if blacklist_games.iter().any(|&r| trimmed_header_text.contains(r)) {
                return Err(format!("{header_text} name in blacklist").into());
            }

            let download_button_selector = scraper::Selector::parse("div.container.content.zero_padding > div.col-lg-10.col-md-12.col-sm-12.col-xs-12 > div.col-lg-8.col-md-8.col-sm-12.col-xs-12.zero > a").unwrap();
            if let Some(pic_href) = document.select(&download_button_selector).next() {
                let href = pic_href.value().attr("href").unwrap();

                let ext = &href[href.len() - 3..];
                let image_name = format!("{header_text}_{pic_index}.{ext}", header_text = header_text, pic_index = pic_index);
                let folder_path = std::path::Path::new(&download_folder);

                if !folder_path.exists() {
                    fs::create_dir_all(folder_path.clone())?;
                }
                let image_path = folder_path.join(image_name.clone());

                download_chunk(href, image_path);

                return Ok(format!("Image {} was saved", image_name));
            } else {
                return Err("Download button selector not found".into());
            }
        } else {
            return Err("Header selector not found".into());
        }
    }
    return Err(format!("{page_url} Page not found").into());
}

async fn download_pictures(
    num_of_pictures: u32,
    resolution: String,
    download_folder: String,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://hdqwalls.com/category/games-wallpapers/{resolution}/sort/views/page/1"
        ))
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
    let pagination_selector =
        scraper::Selector::parse(".pagination_container > div > ul > li:nth-last-child(2)")
            .unwrap();
    let pictures_selector = scraper::Selector::parse("div.wall-resp").unwrap();
    if let Some(last_li) = document.select(&pagination_selector).next() {
        let pages = last_li
            .text()
            .collect::<Vec<_>>()
            .concat()
            .parse::<usize>()
            .unwrap();
        let pictures_per_page = document.select(&pictures_selector).count();
        let max_value = pages * pictures_per_page;

        for _i in 0..num_of_pictures {
            let mut rng = rand::thread_rng(); // create a new Rng instance for each task
            let picture_num = rng.gen_range(0..max_value);
            let page = picture_num / pictures_per_page;
            let pic_index_on_page = picture_num % pictures_per_page;
            match download_image(
                pic_index_on_page,
                page,
                picture_num,
                resolution.clone(),
                download_folder.clone(),
            ).await {
                Ok(mes) => {
                    println!("{}", mes);
                }
                Err(err) => {
                    println!("Box<dyn Error> while downloading image: {}", err);
                }
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
    let window = app.get_window("main").unwrap();
    #[cfg(debug_assertions)] // only include this code on debug builds
    {
        window.open_devtools();
    }
    // This one


    tauri::async_runtime::spawn(async move {
        // also added move here
        match download_pictures(10, "2560x1440".to_string(), "img".to_string()).await {
            Ok(mes) => {
                window.emit("download-progress", 100);
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
        /*.invoke_handler(tauri::generate_handler![js2rs])*/
        .run(tauri::generate_context!())
        .expect("Box<dyn Error> while running tauri application");

    println!("main loop");
}
