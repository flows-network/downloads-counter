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

#[request_handler(GET, POST)]
async fn handler(
    _headers: Vec<(String, String)>,
    _subpath: String,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    logger::init();

    let marker_str = env::var("MARKER_STR").unwrap_or(String::from("download_sh"));

    match _qry.get("marker_str") {
        Some(m) => match serde_json::from_value::<String>(m.clone()) {
            Ok(s) => {
                if s != marker_str {
                    log::error!("invalid marker_str: {}", marker_str);
                    return;
                }
            }
            Err(_e) => {
                log::error!("failed to parse marker_str: {}", _e);
                return;
            }
        },
        _ => {
            log::error!("missing marker_str");
            return;
        }
    }

    let mut download_count = match get("download_count") {
        Some(val) => match serde_json::from_value::<i32>(val) {
            Ok(n) => n,
            Err(_e) => {
                log::error!("failed to parse data from store: {}", _e);
                0
            }
        },
        None => 0,
    };
    download_count += 1;
    set("download_count", serde_json::json!(download_count), None);

    log::error!("Downloads_count: {}", download_count);

    let download_url = "https://raw.githubusercontent.com/second-state/llama-utils/main/run-llm.sh";

    // send_response(
    //     302,
    //     vec![(
    //         String::from("content-type"),
    //         String::from("text/plain; charset=UTF-8"),
    //     )],
    //     format!("Location: {}", download_url).as_bytes().to_vec(),
    // );

    send_response(
        302, // HTTP status code for Found (Redirection)
        vec![
            ("Location".to_string(), download_url.to_string()), // Redirect URL in the Location header
        ],
        Vec::new(), // No body for a redirection response
    );
}
