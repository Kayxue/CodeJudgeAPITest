use ntex::web;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::process::Command;

#[derive(Deserialize)]
struct JudgeResult {
    succeess: bool,
    miliSecond: i32,
}

#[derive(Serialize)]
struct JudgeRequest {
    id: String,
    code: String,
}

#[web::get("/")]
async fn hello() -> impl web::Responder {
    "Welcome to the API"
}

#[web::post("/judge")]
async fn judge() -> impl web::Responder {
    let codes = fs::read_to_string("Test/Test.cpp").unwrap();
    Command::new("g++")
        .arg("Test/Test.cpp")
        .arg("-o")
        .arg("Test/Test")
        .output()
        .unwrap();
    let output = Command::new("./Test/Test.exe").output().unwrap();
    io::stdout().write_all(&output.stdout).unwrap();
    "Compiled Succeed"
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    web::HttpServer::new(|| web::App::new().service(hello).service(judge))
        .bind(("localhost", 80))?
        .run()
        .await
}
