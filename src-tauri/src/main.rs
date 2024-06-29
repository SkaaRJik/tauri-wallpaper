// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate sxd_document;

use std::error::Error;
use std::sync::{Arc, Mutex};

use rand::Rng;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::StatusCode;
use scraper::Html;
use tauri::Manager;

#[tauri::command]
fn my_custom_command() {
    println!("I was invoked from JS!");
}

async fn get_document_html(url: String) -> Result<Html, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .header(ACCEPT, "text/html; charset=utf-8")
        .send()
        .await?;

    let raw_html = match response.status() {
        StatusCode::OK => response.text().await?,
        _ => panic!("Something went wrong"),
    };

    let document = Html::parse_document(&raw_html);
    Ok(document)
}

async fn get_num_of_pages(resolution: String) -> Result<(usize, usize, usize), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://hdqwalls.com/category/games-wallpapers/{resolution}/page/1",
            resolution = resolution,
        ))
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .header(ACCEPT, "text/html; charset=utf-8")
        .send()
        .await?;

    let raw_html = match response.status() {
        StatusCode::OK => response.text().await?,
        _ => panic!("Something went wrong"),
    };

    let document = Html::parse_document(&raw_html);
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
        let max_pictures = pages * pictures_per_page;
        return Ok((pages, pictures_per_page, max_pictures));
    }
    Err(format!("Can't calculate pages").into())
}

async fn get_imgs_links(
    links: Arc<Mutex<Vec<String>>>,
    pages: Arc<u32>,
    resolution: Arc<String>,
    download_folder:  Arc<String>,
) -> Result<String, Box<dyn Error>> {
    loop {
        let pic_page = rand::thread_rng().gen_range(0..*pages);
        let page_url = format!(
            "https://hdqwalls.com/category/games-wallpapers/{resolution}/page/{pic_page}",
            resolution = *resolution,
            pic_page = pic_page
        );

        let document = get_document_html(page_url).await.unwrap();

        let picture_list_selector =
            scraper::Selector::parse("a.caption")
                .unwrap();
        let pictures_per_page = document.select(&picture_list_selector).count();
        let picture_index = rand::thread_rng().gen_range(1..pictures_per_page+1);

        let pictures_selector = scraper::Selector::parse(format!("a.caption:nth-child({})", picture_index).as_str()).unwrap();
        let pic_page_element = document.select(&pictures_selector).next().unwrap();
        let pic_url = pic_page_element.value().attr("href").unwrap().to_string();
        println!("{}", pic_url);
        // Lock the mutex to get access to the vector
        let mut links_guard = links.lock().unwrap();
        if !links_guard.contains(&pic_url) {
            links_guard.push(pic_url.clone());
            println!("added {}", pic_url);
            return Ok("Added".to_string());
        } else {
            return Err("URL already exists".into());
        }
    }
    Err("No picture found".into())
}

async fn download_pictures(num_of_pics: i32, resolution: Arc<String>, img_folder: Arc<String>) -> Result<String, Box<dyn Error>> {
    let links: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let arc_pages = Arc::new(30);

    match get_imgs_links(
        Arc::clone(&links),
        Arc::clone(&arc_pages),
        Arc::clone(&resolution),
        Arc::clone(&img_folder),
    ).await {
        Ok(msg) => println!("{}", msg),
        Err(err) => println!("{}", err),
    }

    Ok("Good".to_string())
}

fn setup<'a>(app: &'a mut tauri::App) -> Result<(), Box<dyn Error>> {
    let window = app.get_window("main").unwrap();
    #[cfg(debug_assertions)] // only include this code on debug builds
    {
        window.open_devtools();
    }

    tauri::async_runtime::spawn(async move {
        // added move here
        let resolution = Arc::new(String::from("2560x1440"));
        let img_folder = Arc::new(String::from("img"));
        match download_pictures(10, resolution, img_folder).await {
            Ok(msg) => {
                window.emit("download-progress", 100).unwrap();
                println!("{}", msg);
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
