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
        Some(m) => match m {
            Value::String(s) => if s == marker_str {},
            _ => return,
        },
        _ => return,
    }

    // bash <(curl -sSf https://raw.githubusercontent.com/second-state/llama-utils/main/run-llm.sh)
    let download_url = "https://raw.githubusercontent.com/second-state/llama-utils/main/run-llm.sh";

    let mut count = get("download_count").unwrap_or(0);
    count = count + 1;
    set("download_count", count, None).unwrap();

    log::error!("Downloads_count: {}", count);

    send_response(
        302,
        vec![(
            String::from("content-type"),
            String::from("text/plain; charset=UTF-8"),
        )],
        download_url.as_bytes().to_vec(),
    );
}
