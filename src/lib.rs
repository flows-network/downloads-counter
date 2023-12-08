use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use store_flows::{get, set};
use webhook_flows::{create_endpoint, request_handler, send_response};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    dotenv().ok();
    create_endpoint().await;
}

async fn create_map() -> HashMap<String, String> {
    let json_contents = include_str!("../URLS.json");

    let urls: Vec<String> = serde_json::from_str(json_contents).expect("failed to parse json");

    let files = env::var("FILES").unwrap_or(String::from(
        "https://raw.githubusercontent.com/second-state/llama-utils/main/run-llm.sh",
    ));

    let mut paths_list = Vec::<String>::new();

    paths_list.extend(urls.into_iter());
    paths_list.extend(files.split_ascii_whitespace().map(String::from));

    paths_list
        .iter()
        .map(|u| {
            let file = u.rsplitn(2, '/').nth(0).unwrap_or(&String::new()).to_string();
            (file, u.clone())
        })
        .collect::<HashMap<String, String>>()
}

#[request_handler(GET, POST)]
async fn handler(
    _headers: Vec<(String, String)>,
    _subpath: String,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    logger::init();

    let urls_map = create_map().await;

    let mut key = String::new();

    match _qry.get("file") {
        Some(m) => match serde_json::from_value::<String>(m.clone()) {
            Ok(s) => {
                if !urls_map.contains_key(&s) {
                    log::error!("invalid file_name: {}", s);
                    return;
                } else {
                    key = s;
                }
            }
            Err(_e) => {
                log::error!("failed to parse file_name: {}", _e);
                return;
            }
        },
        _ => {
            log::error!("missing file_name");
            return;
        }
    }

    let download_url = match urls_map.get(&key) {
        Some(m) => m,
        None => {
            log::error!("missing download_url for file: {}", key);
            return;
        }
    };
    let mut download_count = match get(&key) {
        Some(val) => match serde_json::from_value::<i32>(val) {
            Ok(n) => n,
            Err(_e) => {
                log::error!("failed to parse download_count from store: {}", _e);
                0
            }
        },
        None => 0,
    };
    download_count += 1;
    set("download_count", serde_json::json!(download_count), None);

    log::info!("Downloads_count: {}", download_count);

    send_response(
        302, // HTTP status code for Found (Redirection)
        vec![
            ("Location".to_string(), download_url.to_string()), // Redirect URL in the Location header
        ],
        Vec::new(), // No body for a redirection response
    );
}
