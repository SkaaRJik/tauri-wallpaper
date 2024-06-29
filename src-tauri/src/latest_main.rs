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
use std::sync::{Arc};
use futures_util::stream::StreamExt;
use rand::Rng;
use reqwest;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::StatusCode;
use tauri::{Manager};
use futures::future::join_all;
use scraper::Html;
use tokio::sync::Mutex;

#[tauri::command]
fn my_custom_command() {
    println!("I was invoked from JS!");
}

async fn get_document_html(url: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let response = match client
        .get(url)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .header(ACCEPT, "text/html; charset=utf-8")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return Err(e.to_string().into());
        }
    };

    let raw_html = match response.status() {
        StatusCode::OK => response.text().await.unwrap(),
        _ => return Err(String::from("Something went wrong").into()),
    };
    
    return Ok(raw_html);
}

async fn download_chunk(href: String, image_path: PathBuf) -> Result<String, Box<dyn Error + Send + Sync>> {
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
    resolution: Arc<String>,
    download_folder: Arc<String>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let blacklist_games = [
        "pubg", "playerunknown", "fifa", "nba", "football", "genshin", "fortnite",
    ];


    let page_url = format!(
        "https://hdqwalls.com/category/games-wallpapers/{resolution}/sort/views/page/{pic_page}",
        resolution = resolution,
        pic_page = pic_page
    );

    let pictures_list_document = match get_document_html(page_url).await {
        Ok(document) =>
            Html::parse_document(&document),

        Err(msg) => return Err(msg),
    };

    let pictures_selector = scraper::Selector::parse(format!("div.wall-resp:nth-child({})", pic_index_page).as_str()).unwrap();


    
    
    let pic_page_url = match pictures_list_document.select(&pictures_selector).next() {
        Some(pic_page_element) => 
            String::from(pic_page_element.value().attr("href").unwrap()),
        
        _ => return Err(String::from("Something went wrong").into()),
    };
    
    drop(pictures_selector);
    
    let document = match get_document_html(pic_page_url.clone()).await {
        Ok(document) =>
            Html::parse_document(&document),

        Err(msg) => return Err(msg),
    };
    let header_selector = scraper::Selector::parse("div.page-header > h1").unwrap();

    let header_text = match  document.select(&header_selector).next() {
        Some(header_element) => header_element
            .text()
            .collect::<Vec<_>>()
            .concat()
            .replace("\n", ""),
        _ => return Err(String::from("Something went wrong").into()),
    };
    
    drop(header_selector);
    let trimmed_header_text = header_text.trim().to_lowercase();
    if blacklist_games.iter().any(|&r| trimmed_header_text.contains(r)) {
        return Err(format!("{header_text} name in blacklist").into());
    }
    
    let download_button_selector = scraper::Selector::parse("div.container.content.zero_padding > div.col-lg-10.col-md-12.col-sm-12.col-xs-12 > div.col-lg-8.col-md-8.col-sm-12.col-xs-12.zero > a").unwrap();

    let (pic_href, image_path) = match document.select(&download_button_selector).next() {
        Some(pic_element_href) => {
            let href = String::from(pic_element_href.value().attr("href").unwrap());

            let ext = &href[href.len() - 3..];
            let image_name = format!("{header_text}_{pic_index}.{ext}", header_text = header_text, pic_index = pic_index);
            let folder_path = std::path::Path::new(download_folder.as_str());

            if !folder_path.exists() {
                fs::create_dir_all(folder_path.clone())?;
            }
            let image_path = folder_path.join(image_name);
            
            (href, image_path)
        },
        _ => return Err(String::from("Something went wrong").into()),
    };
    drop(document);
    drop(download_button_selector);
    match download_chunk(pic_href.clone(), image_path.clone()).await {
        Ok(message) => {
            return Ok(format!("Image {} was saved", pic_href)); 
        },
        _ => return Err(String::from("Something went wrong").into()),
    };
    return Err(String::from("Something went wrong").into());
}

async fn download_pictures(
    num_of_pictures: u32,
    resolution: Arc<String>,
    download_folder: Arc<String>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    
    let document = match get_document_html(format!(
        "https://hdqwalls.com/category/games-wallpapers/{resolution}/sort/views/page/1"
    )).await {
        Ok(document) =>
            Html::parse_document(&document),

        Err(msg) => return Err(msg),
    };
    
    let pagination_selector =
        scraper::Selector::parse(".pagination_container > div > ul > li:nth-last-child(2)")
            .unwrap();
    let pictures_selector = scraper::Selector::parse("div.wall-resp").unwrap();
    
    let (pictures_per_page, total_images) = match  document.select(&pagination_selector).next() {
        Some(last_li) => {
            let pages = last_li
                .text()
                .collect::<Vec<_>>()
                .concat()
                .parse::<usize>()
                .expect("Pages not found");
            // let pictures_per_page =  doc_locked.select(&pictures_selector).count();
            let pictures_per_page =  16;
            
            let max_value = pages * pictures_per_page;
            (pictures_per_page, max_value)
        }
        _ => return Err(String::from("Something went wrong").into()),
    };
    drop(document);

    let mut tasks = vec![];
    for _i in 0..num_of_pictures {
        let picture_num = rand::thread_rng().gen_range(0..total_images);
        let page = picture_num / pictures_per_page;
        let pic_index_on_page = picture_num % pictures_per_page;
        tasks.push(download_image(
            pic_index_on_page,
            page,
            picture_num,
            resolution.clone(),
            download_folder.clone(),
        ));
    }

    tauri::async_runtime::spawn(async move {
        let _vec1 = join_all(tasks).await;
    });
    


    Ok("Images downloaded".to_string())
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
        let imgFolder = Arc::new(String::from("img"));
        match download_pictures(10, resolution, imgFolder).await {
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
