use std::collections::HashMap;
use std::str::FromStr;
use std::*;

use io::{Read, Write};
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
macro_rules! line {
    () => {
        |e| format!("{}:{}", panic::Location::caller().line(), e)
    };
}
macro_rules! okline {
    () => {
        || format!("{}", panic::Location::caller().line())
    };
}
struct Post {
    num: i32,
    total: i32,
    user: String,
    id: String,
    atts: Vec<Attachment>,
}
struct Attachment {
    server: String,
    filename: String,
    path: String,
    post: sync::Arc<sync::RwLock<Post>>,
    typ: i32,
}
impl Attachment {
    fn new(att_raw: &serde_json::Value, post: sync::Arc<sync::RwLock<Post>>, typ: i32) -> Result<Self> {
        Ok(Attachment {
            server: att_raw["server"].as_str().ok_or_else(okline!())?.to_string(),
            filename: att_raw["name"].as_str().ok_or_else(okline!())?.to_string(),
            path: att_raw["path"].as_str().ok_or_else(okline!())?.to_string(),
            post,
            typ,
        })
    }
}
fn get_data(url: &str, header: &HashMap<String, String>) -> Result<(Vec<u8>, i32)> {
    let client = reqwest::blocking::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in header {
        let k = key.trim();
        let v = value.trim();
        if v.len() == 0 {
            continue;
        }
        headers.insert(
            reqwest::header::HeaderName::from_str(k).map_err(line!())?,
            reqwest::header::HeaderValue::from_str(v).map_err(line!())?,
        );
    }
    let resp = client.get(url).headers(headers).send().map_err(line!())?;
    let status = resp.status().as_u16() as i32;
    let data = resp.bytes().map_err(line!())?.to_vec();
    Ok((data, status))
}
fn get_json(url: &str, header: &HashMap<String, String>) -> Result<Vec<u8>> {
    let mut tryy = 0;
    loop {
        match get_data(url, header).map_err(line!()) {
            Ok((d, s)) => {
                if s == 200 {
                    return Ok(d);
                }
                if s == 429 {
                    continue;
                }
                if tryy > 5 && s != 200 {
                    return Err(format!("status-code:{}:{}", s, String::from_utf8(d).map_err(line!())?).into());
                }
            }
            Err(e) => {
                if tryy > 5 {
                    return Err(e.into());
                }
            }
        };
        tryy += 1;
    }
}
fn get_post_from_search(
    search_term: &str,
    _platform: &str,
    base_url: &str,
    start: i32,
    end: i32,
    preset: i32,
    header: &HashMap<String, String>,
) -> Result<Vec<sync::Arc<sync::RwLock<Post>>>> {
    let mut posts: Vec<sync::Arc<sync::RwLock<Post>>> = Vec::new();
    'out: {
        let mut n = (start / 50) * 50;
        let mut total = n;
        while n <= total {
            print!("\033[2KDownloading Page:{}\n\033[A", (n / 50) + 1);
            let url = format!("https://{}/api/v1/posts?q={}&o={}", base_url, search_term, n);
            let json_data = get_json(url.as_str(), header).map_err(line!())?;
            let json: serde_json::Value = serde_json::from_slice(&json_data).map_err(line!())?;
            total = json["count"].as_i64().ok_or_else(okline!())? as i32;
            let posts_raw = json["posts"].as_array().ok_or_else(okline!())?;
            for (m, post_raw) in posts_raw.iter().enumerate() {
                if n + 1 + (m as i32) < start {
                    continue;
                }
                let id = post_raw["id"].as_str().ok_or_else(okline!())?.to_string();
                let user = post_raw["user"].as_str().ok_or_else(okline!())?.to_string();
                let service = post_raw["service"].as_str().ok_or_else(okline!())?.to_string();
                let url = format!("https://{}/api/v1/{}/user/{}/post/{}", base_url, service, user, id);
                let json_data = get_json(url.as_str(), header).map_err(line!())?;
                let json: serde_json::Value = serde_json::from_slice(&json_data).map_err(line!())?;
                let post_sync = sync::Arc::new(sync::RwLock::new(Post {
                    id,
                    user,
                    total,
                    num: n + 1 + (m as i32),
                    atts: Vec::new(),
                }));
                let mut post = match post_sync.write() {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                if preset == 0 || preset == 1 {
                    let j = json["attachments"].as_array().ok_or_else(okline!())?;
                    for a in j {
                        post.atts.push(Attachment::new(a, post_sync.clone(), 1).map_err(line!())?);
                    }
                }
                if preset == 0 || preset == 2 {
                    let j = json["previews"].as_array().ok_or_else(okline!())?;
                    for a in j {
                        post.atts.push(Attachment::new(a, post_sync.clone(), 2).map_err(line!())?);
                    }
                }
                if post.atts.len() == 0 {
                    continue;
                }
                drop(post);
                posts.push(post_sync);
                if n + 1 + (m as i32) >= end {
                    if end != 0 {
                        break 'out;
                    }
                }
            }
            n += 50;
        }
    }
    println!();
    Ok(posts)
}
fn get_post_from_profile(
    model: &str,
    platform: &str,
    base_url: &str,
    start: i32,
    end: i32,
    preset: i32,
    header: &HashMap<String, String>,
) -> Result<Vec<sync::Arc<sync::RwLock<Post>>>> {
    let mut posts: Vec<sync::Arc<sync::RwLock<Post>>> = Vec::new();
    'out: {
        let mut n = (start / 50) * 50;
        let mut total = n;
        while n <= total {
            print!("\x1b[2KDownloading Page:{}\n\x1b[A", (n / 50) + 1);
            let url = format!("https://{}/api/v1/{}/user/{}/posts-legacy?o={}", base_url, platform, model, n);
            let json_data = get_json(url.as_str(), header).map_err(line!())?;
            let json: serde_json::Value = serde_json::from_slice(&json_data).map_err(line!())?;
            total = json["props"]["count"].as_i64().ok_or_else(okline!())? as i32;
            let results = json["results"].as_array().ok_or_else(okline!())?;
            for (m, result) in results.iter().enumerate() {
                if n + 1 + (m as i32) < start {
                    continue;
                }
                let post_sync = sync::Arc::new(sync::RwLock::new(Post {
                    id: result["id"].as_str().ok_or_else(okline!())?.to_string(),
                    user: result["user"].as_str().ok_or_else(okline!())?.to_string(),
                    total,
                    num: n + 1 + (m as i32),
                    atts: Vec::new(),
                }));
                let mut post = match post_sync.write() {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                if preset == 0 || preset == 1 {
                    let j = json["result_attachments"].as_array().ok_or_else(okline!())?[m]
                        .as_array()
                        .ok_or_else(okline!())?;
                    for a in j {
                        post.atts.push(Attachment::new(a, post_sync.clone(), 1).map_err(line!())?);
                    }
                }
                if preset == 0 || preset == 2 {
                    let j = json["result_previews"].as_array().ok_or_else(okline!())?[m]
                        .as_array()
                        .ok_or_else(okline!())?;
                    for a in j {
                        post.atts.push(Attachment::new(a, post_sync.clone(), 2).map_err(line!())?);
                    }
                }
                if post.atts.len() == 0 {
                    continue;
                }
                drop(post);
                posts.push(post_sync);
                if n + 1 + (m as i32) >= end {
                    if end != 0 {
                        break 'out;
                    }
                }
            }
            n += 50;
        }
    }
    println!();
    Ok(posts)
}
fn save_media_loop(filename: String, media_link: String) -> Result<()> {
    let mut i = 0;
    loop {
        match save_media(&filename, &media_link).map_err(line!()) {
            Ok(_) => return Ok(()),
            Err(e) => {
                if i > 100 {
                    return Err(e).map_err(line!())?;
                }
            }
        };
        i += 1
    }
}
fn save_media(filename: &String, media_link: &String) -> Result<()> {
    let mut size = 0;
    match fs::metadata(filename) {
        Ok(metadata) => {
            if metadata.is_file() {
                size = metadata.len();
            }
        }
        _ => (),
    }
    loop {
        let resp = reqwest::blocking::get(media_link).map_err(line!())?;
        if resp.status().as_u16() == 429 {
            continue;
        }
        if resp.status().as_u16() != 200 {
            return Err(resp.status().as_str()).map_err(line!())?;
        }
        let fullsize = resp
            .headers()
            .get("content-length")
            .ok_or_else(okline!())?
            .to_str()
            .map_err(line!())?
            .trim()
            .parse::<u64>()
            .map_err(line!())?;
        if size >= fullsize {
            return Ok(());
        }
        break;
    }
    let mut resp: reqwest::blocking::Response;
    loop {
        resp = reqwest::blocking::Client::new()
            .get(media_link)
            .header("Range", format!("bytes={}-", size))
            .send()
            .map_err(line!())?;
        if resp.status().as_u16() == 429 {
            continue;
        }
        if resp.status().as_u16() != 200 && resp.status().as_u16() != 206 {
            return Err(resp.status().as_str()).map_err(line!())?;
        }
        break;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename)
        .map_err(line!())?;
    let mut buffer: Vec<u8> = vec![0; 1000000];
    let buf = buffer.as_mut_slice();
    loop {
        let n = resp.read(buf).map_err(line!())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(line!())?;
    }
    return Ok(());
}
fn download_content(posts: Vec<sync::Arc<sync::RwLock<Post>>>) -> Result<Vec<String>> {
    let mut server_map: HashMap<String, Vec<Attachment>> = HashMap::new();
    for post_sync in &posts {
        let post = match post_sync.read() {
            Ok(r) => r,
            Err(_) => continue,
        };
        if post.atts.len() == 0 {
            continue;
        }
        for att in &post.atts {
            if !server_map.contains_key(&att.server) {
                server_map.insert(att.server.clone(), Vec::new());
            }
            server_map.get_mut(&att.server).ok_or_else(okline!())?.push(Attachment {
                server: att.server.clone(),
                filename: att.filename.clone(),
                path: att.path.clone(),
                post: att.post.clone(),
                typ: att.typ,
            });
        }
    }
    let mutex_errs = sync::Arc::new(sync::Mutex::new(Vec::new()));
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    for (_, list) in server_map {
        let mutex_errs = sync::Arc::clone(&mutex_errs);
        handles.push(thread::spawn(move || {
            for (n, a) in list.iter().enumerate() {
                let m = format!("{}/data{}", a.server, a.path);
                let post = match a.post.read() {
                    Ok(r) => r,
                    Err(_) => {
                        continue;
                    }
                };
                let f = format!("{}-{}-{}", post.user, post.id, a.filename);
                println!("{}/{}\t{}/{}\t{}", post.num, post.total, n + 1, list.len(), f);
                let err = match save_media_loop(f, m) {
                    Err(e) => e.to_string(),
                    _ => "".to_string(),
                };
                if err.len() > 0 {
                    match mutex_errs.lock().map_err(line!()) {
                        Ok(e) => e,
                        Err(_) => continue,
                    }
                    .push(err);
                }
            }
        }));
    }
    for handle in handles {
        _ = handle.join();
    }
    return Ok(mutex_errs.lock().map_err(line!())?.to_vec());
}
fn main() {
    const TAG: Option<&str> = option_env!("TAG");
    println!("Coomer Download: {}", TAG.unwrap_or_default());
    println!("Enter Header Cookie: ");
    let mut cookie = String::new();
    std::io::stdin().read_line(&mut cookie).map_err(line!()).unwrap();
    println!("Enter Header User-Agent: ");
    let mut user_agent = String::new();
    std::io::stdin().read_line(&mut user_agent).map_err(line!()).unwrap();
    let mut header: HashMap<String, String> = HashMap::new();
    header.insert("cookie".to_string(), cookie);
    header.insert("user-agent".to_string(), user_agent);
    println!("Enter Coomer|Kemono Url:");
    let mut url = String::new();
    std::io::stdin().read_line(&mut url).map_err(line!()).unwrap();
    let url_split = url.split("/").collect::<Vec<&str>>();
    let base_url: &str;
    let mut search_term: &str;
    let mut service = "";
    let mut choice_func: fn(&str, &str, &str, i32, i32, i32, &HashMap<String, String>) -> Result<Vec<sync::Arc<sync::RwLock<Post>>>> =
        get_post_from_profile;
    match url_split.len() {
        4 => {
            choice_func = get_post_from_search;
            base_url = url_split[2];
            search_term = url_split[3];
            match search_term.find("q=") {
                Some(i) => search_term = &search_term[i + 2..],
                _ => {
                    println!("Invalid Url");
                    process::exit(1);
                }
            }
        }
        6 => {
            base_url = url_split[2];
            service = url_split[3];
            search_term = url_split[5];
            match search_term.find("?") {
                Some(i) => {
                    search_term = &search_term[..i];
                }
                _ => {}
            }
        }
        _ => {
            println!("Invalid Url");
            process::exit(1);
        }
    }
    println!("(Press ENTER to Skip the folloing prompts)");
    println!("Enter 1 to download videos only, Enter 2 to download pictures only, Enter 0 for both");
    let mut preset = String::new();
    std::io::stdin().read_line(&mut preset).map_err(line!()).unwrap();
    let mut preset_int = preset.trim().parse::<i32>().unwrap_or(0);
    if preset_int > 2 || preset_int < 0 {
        preset_int = 0;
    }
    println!("Enter starting post number");
    let mut start = String::new();
    std::io::stdin().read_line(&mut start).map_err(line!()).unwrap();
    let start_int = start.trim().parse::<i32>().unwrap_or(1);
    println!("Enter ending post number");
    let mut end = String::new();
    std::io::stdin().read_line(&mut end).map_err(line!()).unwrap();
    let end_int = end.trim().parse::<i32>().unwrap_or(0);
    let posts = choice_func(search_term, service, base_url, start_int, end_int, preset_int, &header)
        .map_err(line!())
        .unwrap();
    let errs = download_content(posts).map_err(line!()).unwrap();
    if errs.len() > 0 {
        println!("Errors:");
        for e in errs {
            eprintln!("{}", e);
        }
        process::exit(1);
    }
    println!("Done");
}
