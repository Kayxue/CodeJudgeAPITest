use ntex::web;
use ntex_cors::Cors;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::str;

#[derive(Serialize)]
struct JudgeResult {
    success: bool,
    miliSecond: i32,
    result: String,
}

#[derive(Debug)]
struct CompileError {
    error: String,
}

#[derive(Deserialize)]
struct JudgeRequest {
    id: String,
    code: String,
}

#[web::get("/")]
async fn hello() -> impl web::Responder {
    "Welcome to the API"
}

#[web::post("/judge")]
async fn judge() -> Result<web::HttpResponse, web::Error> {
    //let codes = fs::read_to_string("Test/Test.cpp").unwrap();
    let compileResult = Command::new("g++")
        .arg("Test/Test.cpp")
        .arg("-o")
        .arg("Test/Test")
        .output()
        .unwrap();
    let mut success = true;
    let compileError = match str::from_utf8(&compileResult.stderr) {
        Ok(val) => val,
        Err(_) => "",
    };
    if compileError.len() != 0 {
        let err = Err(CompileError {
            error: compileError.to_string(),
        });
        return err.map_err(|err| {
            web::error::ErrorBadRequest(err.error).into()
        });
    }
    let output = Command::new("./Test/Test.exe").output().unwrap();
    let mut results = String::new();
    results.push_str(match str::from_utf8(&output.stdout) {
        Ok(val) => val,
        Err(_) => panic!("Runtime Error"),
    });
    Ok(web::HttpResponse::Ok().json(&JudgeResult {
        success: true,
        miliSecond: 1,
        result: results,
    }))
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    web::HttpServer::new(|| {
        web::App::new()
            .wrap(
                Cors::new()
                    .allowed_origin("*")
                    .allowed_methods(vec!["GET", "POST"])
                    .finish(),
            )
            .service(hello)
            .service(judge)
    })
    .bind(("localhost", 80))?
    .run()
    .await
}
