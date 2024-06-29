// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate sxd_document;

use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::StatusCode;
use std::sync::Arc;
use futures_util::future::join_all;
use futures_util::lock::Mutex;
use rand::Rng;
use reqwest;
use tauri::Manager;

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
        let max_pictures = pages * pictures_per_page;
        return Ok((pages, pictures_per_page, max_pictures));
    }
    Err(format!("Can't calculate pages").into())
}
async fn get_imgs_links(
    links: Arc<Mutex<Vec<String>>>,
    pages: Arc<u32>,
    resolution: String,
    download_folder: String,
) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    /*loop {*/
        
        let pic_page = rand::thread_rng().gen_range(0..*pages);
        let page_url = format!(
            "https://hdqwalls.com/category/games-wallpapers/{resolution}/page/{pic_page}",
            resolution = resolution,
            pic_page = pic_page
        );

        let document = get_document_html(&client, page_url.clone()).await.unwrap();

        let pictures_selector = scraper::Selector::parse(format!("a.caption:nth-child({})", 1).as_str()).unwrap();

        if let Some(pic_page_element) = document.select(&pictures_selector).next() {
            let pic_url = pic_page_element.value().attr("href").unwrap();
            println!("{}", pic_url);
            let mut links_guard = links.lock().await;
            if !links_guard.contains(&pic_url.to_string()) {
                links_guard.push(pic_url.to_string());
                println!("added {}", pic_url);
                return Ok("Added".to_string());
            } else {
                return Err(format!("Can't calculate pages").into());
            }
        }
    return Err(format!("Can't calculate pages").into());
    /*}*/
   
}

async fn download_pictures(num_of_pics: i32, resolution: Arc<String>, img_folder: Arc<String>) -> Result<String, Box<dyn Error>> {
    /*match get_num_of_pages(resolution.clone().to_string()).await {
        Ok((pages, pictures_per_page, max_value)) => {

        }
        Err(err) => {
            println!("{}", err.to_string())
        }
    }*/


    let links: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let arcPages = Arc::new(30);
    
    /*let arcPages: Arc<usize> = Arc::new(pages);*/

    
    match get_imgs_links(
        links.clone(),
        Arc::clone(&arcPages),
        resolution.clone().to_string(),
        img_folder.clone().to_string(),
    ).await {
        Ok(str) => {
            println!("{}", str.to_string())
        }
        Err(err) => {
            println!("{}", err.to_string())
        }
    }
    /*
    let mut tasks = vec![];
    for _i in 0..num_of_pics {
        tasks.push(
           get_imgs_links(
                links.clone(),
                Arc::clone(&arcPages),
                resolution.clone().to_string(),
                img_folder.clone().to_string(),
            )
        )
    }

    let res = join_all(tasks).await;*/

    Ok("Good".to_string())
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
        let resolution = Arc::new(String::from("2560x1440"));
        let img_folder = Arc::new(String::from("img"));
        match download_pictures(10, resolution, img_folder).await {
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
