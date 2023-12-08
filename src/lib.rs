use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use std::collections::HashMap;
use webhook_flows::{
    create_endpoint, request_handler,
    route::{get, route, RouteError, Router},
    send_response,
};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    dotenv().ok();
    create_endpoint().await;
}

#[request_handler(GET)]
async fn handler(
    _headers: Vec<(String, String)>,
    _subpath: String,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    logger::init();

    let mut router = Router::new();
    router
        .insert("/redir/:file_name", vec![get(track_and_redirect)])
        .unwrap();

    router
        .insert("/count/:file_name", vec![get(get_download_counts)])
        .unwrap();

    if let Err(e) = route(router).await {
        match e {
            RouteError::NotFound => {
                send_response(404, vec![], b"No route matched".to_vec());
            }
            RouteError::MethodNotAllowed => {
                send_response(405, vec![], b"Method not allowed".to_vec());
            }
        }
    }
}

async fn track_and_redirect(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    let urls_map = create_map().await;

    match _qry.get("file_name") {
        Some(m) => match serde_json::from_value::<String>(m.clone()) {
            Ok(key) => match urls_map.contains_key(&key) {
                true => {
                    let download_url = match urls_map.get(&key) {
                        Some(u) => u,
                        None => {
                            log::error!("missing download_url for file: {}", key);
                            return;
                        }
                    };
                    let download_count = match store_flows::get(&key) {
                        Some(val) => match serde_json::from_value::<i32>(val) {
                            Ok(n) => n + 1,
                            Err(_e) => {
                                log::error!("failed to parse download_count from store: {}", _e);
                                1
                            }
                        },
                        None => 1,
                    };
                    store_flows::set(&key, serde_json::json!(download_count), None);

                    log::info!("{} downloaed {} times", key, download_count);

                    send_response(
                        302, // HTTP status code for Found (Redirection)
                        vec![
                            ("Location".to_string(), download_url.to_string()), // Redirect URL in the Location header
                        ],
                        Vec::new(), // No body for a redirection response
                    );
                }

                false => {
                    log::error!("invalid file_name: {}", key);
                    return;
                }
            },
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
}

async fn get_download_counts(
    _headers: Vec<(String, String)>,
    qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    match qry.get("file_name") {
        Some(m) => match serde_json::from_value::<String>(m.clone()) {
            Ok(file_name) => {
                let download_count = match store_flows::get(&file_name) {
                    Some(val) => match serde_json::from_value::<i32>(val) {
                        Ok(n) => n,
                        Err(_e) => {
                            log::error!("Error parsing download count for {}: {}", file_name, _e);
                            0
                        }
                    },
                    None => 0,
                };
                send_response(
                    200,
                    vec![(String::from("content-type"), String::from("text/html"))],
                    format!("{} has been downloaded {} times", file_name, download_count)
                        .as_bytes()
                        .to_vec(),
                );
            }
            Err(_e) => {
                log::error!("failed to parse file_name from query: {}", _e);
                return;
            }
        },
        _ => {
            log::error!("Failed to find any file_name to query");
            return;
        }
    }
}

async fn create_map() -> HashMap<String, String> {
    let json_contents = include_str!("../URLS.json");
    let mut paths_list = Vec::<String>::new();

    match serde_json::from_str::<Vec<String>>(json_contents) {
        Ok(urls) => paths_list.extend(urls.into_iter()),
        Err(_e) => log::error!("failed to parse URLS.json: {}", _e),
    };

    paths_list
        .iter()
        .filter_map(|u| {
            if let Some(file) = u.rsplitn(2, '/').nth(0) {
                Some((file.to_string(), u.clone()))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>()
}
