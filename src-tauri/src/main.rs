// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod services;

extern crate sxd_document;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::sync::{Arc};
use std::sync::atomic::{AtomicU8, Ordering};
use futures::stream::FuturesUnordered;
use futures::prelude::*;
use futures_util::never::Never;
use rand::{Rng};
use std::io::Write;
use std::string::ToString;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::StatusCode;
use scraper::Html;
use tauri::{Manager, Window};
use chrono::{Utc, DateTime, NaiveDate, Datelike, Timelike};
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use tokio::sync::Mutex;

#[tauri::command]
fn my_custom_command() {
    println!("I was invoked from JS!");
}

#[derive(Clone)]
struct ShortImgInfo {
    title: Arc<String>,
    page_url: Arc<String>,
}

#[derive(Clone)]
struct ImgInfo {
    title: Arc<String>,
    page_url: Arc<String>,
    download_url: Arc<String>,
    download_date: Arc<DateTime<Utc>>,
}

/*enum DownloadSlots {
    DownloadStatus("download-status"),
    SecondVariant,
    ThirdVariant,
}*/
async fn get_document_html(url: String) -> Result<Html, String> {
    let client = reqwest::Client::new();
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

    let document =Html::parse_document(&raw_html);
    
    Ok(document)
}


async fn get_num_of_pages(resolution: Arc<String>) -> Result<u32, Box<dyn Error>> {
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
    if let Some(last_li) = document.select(&pagination_selector).next() {
        let pages = last_li
            .text()
            .collect::<Vec<_>>()
            .concat()
            .parse::<usize>()
            .unwrap();
        return Ok(pages as u32);
    }
    Err(format!("Can't calculate pages").into())
}

fn get_now_date_in_milis() -> u64 {
    let now = Utc::now();
    let naive_date = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
    let timestamp = naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
    println!("Current date in milliseconds: {}", timestamp);
    timestamp as u64
}
async fn get_imgs_links(
    links: Arc<Mutex<Vec<ShortImgInfo>>>,
    pages: Arc<u32>,
    resolution: Arc<String>,
    black_list: Arc<Vec<String>>,
    mut rng: ChaCha8Rng,
) -> Result<String, Never> {
    loop {
        let pic_page = rng.gen_range(0..=*pages);
        
        
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
        
        
        let picture_index = rng.gen_range(2..pictures_per_page + 2);
        println!("Pic index {}", picture_index.clone());
        let pictures_selector = scraper::Selector::parse(format!("div:nth-child({}) > a.caption", picture_index).as_str()).unwrap();
        let pic_page_element = document.select(&pictures_selector).next().unwrap();

        let pic_title = pic_page_element.clone().text().collect::<Vec<_>>().concat().to_lowercase();
        println!("{}", pic_title);
        let arc_black_list = Arc::clone(&black_list);

        let mut repeat_loop = false;

        for black_word in arc_black_list.iter() {
            if pic_title.contains(black_word) {
                repeat_loop = true;
                println!("Loop again because blacklist");
            }
        }

        if (repeat_loop) {
            continue;
        }

        let pic_url = pic_page_element.value().attr("href").unwrap().to_string();
        println!("{}", pic_url.clone());
        // Lock the mutex to get access to the vector
        let mut links_guard = loop {
             if let Ok(lock) = links.clone().try_lock_owned() { break lock }
        };
        

        for link_info in links_guard.iter() {
            let page_url = Arc::clone(&link_info.page_url);
            if page_url.eq_ignore_ascii_case(&pic_url) {
                repeat_loop = true;
                println!("Loop again because duplicate");
            }
        }
        if (repeat_loop) {
            continue;
        }


        links_guard.push(ShortImgInfo {
            title: Arc::new(pic_title),
            page_url: Arc::new(pic_url),
        });
        return Ok("Added".to_string());
    }
}
async fn download_chunk(img_info: Arc<ImgInfo>, download_folder: Arc<String>) -> Result<String, Never> {
    let client = reqwest::Client::new();
    let download_url = Arc::clone(&img_info.download_url);
    let header_text = Arc::clone(&img_info.title);
    let response = client.get(download_url.to_string()).send().await.unwrap();
    let ext = &download_url[download_url.len() - 3..];
    let image_name = format!("{header_text}.{ext}", header_text = header_text.to_string());
    let folder_path = std::path::Path::new(download_folder.as_str());

    if !folder_path.exists() {
        fs::create_dir_all(folder_path).unwrap();
    }
    let image_path = folder_path.join(image_name);


    let mut out = File::create(image_path).unwrap();
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.unwrap();
        out.write_all(&chunk).unwrap();
    }
    out.flush().unwrap();
    Ok(format!("{} downloaded", img_info.title.clone().to_string()))
}

async fn _download_chunk() -> Result<String, Never> {
    /*let client = reqwest::Client::new();
    let download_url = Arc::clone(&img_info.download_url);
    let header_text = Arc::clone(&img_info.title);*/
    /*let response = client.get(download_url.to_string()).send().await.unwrap();
    let ext = &download_url[download_url.len() - 3..];
    let image_name = format!("{header_text}.{ext}", header_text = header_text.to_string());
    let folder_path = std::path::Path::new(download_folder.as_str());

    if !folder_path.exists() {
        fs::create_dir_all(folder_path.clone()).unwrap();
    }
    let image_path = folder_path.join(image_name);


    let mut out = File::create(image_path).unwrap();
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.unwrap();
        out.write_all(&chunk).unwrap();
    }
    out.flush().unwrap();
    println!("Загрузка завершена");*/
    Ok("Good".to_string())
}

async fn download_image(
    page_info: Arc<ShortImgInfo>,
    download_folder: Arc<String>,
) -> Result<Arc<ImgInfo>, String> {
    
    let pic_page_url = Arc::new(&page_info.page_url);
    let picture_page_document = get_document_html(pic_page_url.to_string()).await.unwrap();
    
    let download_button_selector = scraper::Selector::parse("div.container.content.zero_padding > div.col-lg-10.col-md-12.col-sm-12.col-xs-12 > div.col-lg-8.col-md-8.col-sm-12.col-xs-12.zero > a").unwrap();
   


    let full_pic_url = match picture_page_document.select(&download_button_selector).next() {
        Some(pic_page_element) =>
            String::from(pic_page_element.value().attr("href").unwrap()),

        _ => return Err(String::from("Can't find download button").into()),
    };
    
    
    let updated_struct = Arc::new(ImgInfo {
        title: Arc::clone(&page_info.title),
        page_url: Arc::clone(&page_info.page_url),
        download_url: Arc::new(full_pic_url),
        download_date: Arc::new(Utc::now()),
    });
    
    // return Ok(updated_struct);
    return Ok(updated_struct);
}

async fn download_pictures(num_of_pics: i32, resolution: Arc<String>, img_folder: Arc<String>, black_list: Arc<Vec<String>>, _w: &Window) -> Result<String, Box<dyn Error>> {
    let links: Arc<Mutex<Vec<ShortImgInfo>>> = Arc::new(Mutex::new(vec![]));

    let pages = get_num_of_pages(Arc::clone(&resolution)).await.unwrap();

    let arc_pages = Arc::new(pages);
    println!("pages: {}", Arc::clone(&arc_pages));
    let mut tasks = vec![];
    let SEED = get_now_date_in_milis();
    for _i in 0..num_of_pics {
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        rng.set_stream(_i as u64);
        tasks.push(get_imgs_links(
            Arc::clone(&links),
            Arc::clone(&arc_pages),
            Arc::clone(&resolution),
            Arc::clone(&black_list),
            rng
        ));
    }

    let mut atomic = AtomicU8::new(1);
    let mut completion_stream = tasks.into_iter()
        .map(tokio::spawn)
        .collect::<FuturesUnordered<_>>();
    
    let part_of_percentage = (50 / num_of_pics) as u8;
    
    while let Some(res) = completion_stream.next().await {
        let _r = res.unwrap().unwrap();
        let ind = atomic.fetch_add(1, Ordering::SeqCst);
        let progress = ind * part_of_percentage;
        _w.emit("download-progress", progress).unwrap();
        println!("Some {}", progress);
    }


    let links_guard = loop {
        if let Ok(lock) = links.clone().try_lock_owned() { break lock }
    };

    
    let mut tasks2 = vec![];
    for info in links_guard.iter() {
        tasks2.push(download_image(
            Arc::new(info.clone()),
            Arc::clone(&img_folder)
        ));
    }

    let mut completion_stream_2 = tasks2.into_iter()
        .map(tokio::spawn)
        .collect::<FuturesUnordered<_>>();

    atomic = AtomicU8::new(1);
    while let Some(res) = completion_stream_2.next().await {
        let img_info = res.unwrap().unwrap();
        let download_result = download_chunk(Arc::clone(&img_info), Arc::clone(&img_folder)).await.unwrap();
        let ind = atomic.fetch_add(1, Ordering::SeqCst);
        let progress = 50 + ind * part_of_percentage;
        _w.emit("download-progress", ind * 5).unwrap();
        println!("{progress}% - {download_result}", download_result = download_result, progress = progress);
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
        let black_list = Arc::new(vec![
            "pubg",
            "Solo Leveling Arise",
            "playerunknown",
            "fifa",
            "garena",
            "nba",
            "football",
            "genshin",
            "soccer",
            "hockey",
            "fortnite",
        ]
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<String>>());
        match download_pictures(10, resolution, img_folder, black_list, &window).await {
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
