use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::{Server, Response, Method, Header, StatusCode};
use serde::{Deserialize, Serialize};

use predict::AI;

#[derive(Deserialize)]
struct ChatRequest {
    prompt: String,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
}

fn main() -> std::io::Result<()> {
    // Create a shared AI instance
    let ai = Arc::new(Mutex::new(AI::new("weights/model_int4.bin")));

    let server = match Server::http("0.0.0.0:3030") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to bind server: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("server bind error: {}", e)));
        }
    };
    println!("Server running on http://0.0.0.0:3030");

    for request in server.incoming_requests() {
        let ai = ai.clone();
        let mut req = request;
        // Spawn a thread per request to keep responsiveness
        thread::spawn(move || {
            let url = req.url().to_string();
            let method = req.method().clone();

            // simple health
                if method == Method::Get && url == "/health" {
                let mut response = Response::from_string("OK");
                response.add_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = req.respond(response);
                return;
            }

            if method == Method::Post && url == "/chat" {
                // read body
                let mut content = String::new();
                if let Ok(_) = req.as_reader().read_to_string(&mut content) {
                    if let Ok(chat_req) = serde_json::from_str::<ChatRequest>(&content) {
                        // call AI
                        let reply = {
                            let mut ai = ai.lock().unwrap();
                            ai.chat(&chat_req.prompt)
                        };
                        let body = serde_json::to_string(&ChatResponse { reply }).unwrap();
                        let mut response = Response::from_string(body);
                        response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                        response.add_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                        let _ = req.respond(response);
                        return;
                    }
                }
                let mut r = Response::from_string("Bad Request");
                r = r.with_status_code(StatusCode(400));
                let _ = req.respond(r);
                return;
            }

            // unsupported
            let mut r = Response::from_string("Not Found");
            r = r.with_status_code(StatusCode(404));
            let _ = req.respond(r);
        });
    }

    Ok(())
}
