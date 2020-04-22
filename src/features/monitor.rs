//! Monitor our web services, and the swarm using the Traefik Health informations.

use core::connection::CHANNEL_MAIN;
use reqwest;
use serde_json::{from_str, Value};
use std::{thread, time};

/// Simple get request, returned as string
fn request() -> String {
    reqwest::get("http://jungleruns.com:8080/health")
        .unwrap()
        .text()
        .unwrap()
}

/// Request the codes and parse them, returning a `Value`
fn get_codes() -> Value {
    let response = request();
    let v: Value = from_str(&response).unwrap();
    let codes = v["total_status_code_count"].clone();
    codes
}

pub fn display_codes(_args: &[&str]) -> String {
    get_codes().to_string()
}

/// Main loop on a fixed time, get, parse and check for the error codes.
pub fn error_code_check() {
    println!("Health check thread started");

    let mut error_code_number: i64 = <i64>::max_value();
    // let mut notfound_code_number: i64 = <i64>::max_value();

    loop {
        let mut display_codes = false;
        let codes = get_codes();
        let number_tmp = codes["500"].as_i64().unwrap();
        if error_code_number < number_tmp {
            display_codes = true;
            let msg = format!(
                "Error code 500 going up ! {} -> {}",
                error_code_number, number_tmp
            );
            let _ = CHANNEL_MAIN.send_message(|m| m.content(msg));
        }
        // let number_tmp = codes["404"].as_i64().unwrap();
        // if notfound_code_number < number_tmp {
        //     display_codes = true;
        //     let msg = format!(
        //         "Error code 404 going up ! {} -> {}",
        //         notfound_code_number, number_tmp
        //     );
        //     let _ = CHANNEL_MAIN.send_message(|m| m.content(msg));
        // }

        if display_codes == true {
            let _ = CHANNEL_MAIN.send_message(|m| m.content(format!("Codes = {}", codes)));
        }
        error_code_number = codes["500"].as_i64().unwrap();
        // notfound_code_number = codes["404"].as_i64().unwrap();
        thread::sleep(time::Duration::from_millis(1000 * 60 * 10));
    }
}
