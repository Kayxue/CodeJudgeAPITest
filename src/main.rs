#![allow(non_snake_case)]
use ntex::web;
use ntex_cors::Cors;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{process::Command, str, time::Instant};

#[derive(Serialize)]
struct JudgeResult {
    success: bool,
    miliSecond: u128,
    result: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum JudgeErrorType {
    Compile,
    Runtime,
}

impl JudgeErrorType {
    fn as_str(&self) -> &'static str {
        match self {
            JudgeErrorType::Compile => "Compile",
            JudgeErrorType::Runtime => "Runtime",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct JudgeError {
    errorType: JudgeErrorType,
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
    //Read local file
    //let codes = fs::read_to_string("Test/Test.cpp").unwrap();

    //Try to compile file with g++
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
        let err = Err(JudgeError {
            errorType: JudgeErrorType::Compile,
            error: compileError.to_string(),
        });
        return err.map_err(|err| {
            web::error::ErrorBadRequest(serde_json::to_string(&err).unwrap()).into()
        });
    }

    //Execute Program
    let startTime = Instant::now();
    let output = Command::new("./Test/Test.exe").output().unwrap();
    let duration = startTime.elapsed();
    let (runSucceed, statusCode) = (
        output.status.success(),
        match output.status.code() {
            Some(code) => code,
            None => 0,
        },
    );
    if !runSucceed {
        let err = Err(JudgeError {
            errorType: JudgeErrorType::Runtime,
            error: format!("Program exited with code {}", statusCode),
        });
        return err.map_err(|err| {
            web::error::ErrorBadRequest(serde_json::to_string(&err).unwrap()).into()
        });
    }
    let mut results = String::new();
    results.push_str(match str::from_utf8(&output.stdout) {
        Ok(val) => val,
        Err(_) => panic!("Runtime Error"),
    });
    Ok(web::HttpResponse::Ok().json(&JudgeResult {
        success: true,
        miliSecond: duration.as_millis(),
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
