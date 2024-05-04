#![allow(non_snake_case)]
// #![allow(dead_code)]
use futures::{Stream, StreamExt, TryStreamExt};
use mp::Multipart;
use ntex::web::{self, Error};
use ntex_cors::Cors;
use ntex_multipart as mp;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    io::Read,
    process::{Command, Stdio},
    str,
    time::{Duration, Instant},
};
use uuid::Uuid;
use wait_timeout::ChildExt;

#[derive(Serialize)]
struct JudgeResult {
    success: bool,
    miliSecond: u128,
    result: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JudgeInformation {
    code: String,
    language: Languages,
    question: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum Languages {
    cpp,
    java,
    rust,
}

#[derive(Debug, Serialize, Deserialize)]
enum JudgeErrorType {
    Compile,
    Runtime,
}

#[derive(Debug, Deserialize, Serialize)]
struct JudgeError {
    errorType: JudgeErrorType,
    error: String,
}

#[web::get("/")]
async fn hello() -> impl web::Responder {
    "Welcome to the API"
}

async fn saveFile(mut payload: mp::Multipart, uuid: &str) -> Result<JudgeInformation, bool> {
    let mut infoJson = String::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        println!("{}", field.content_type().to_string());
        if field.content_type().to_string() == "application/octet-stream" {
            let chunk = field.next().await;
            let data = chunk.unwrap().unwrap();
            // println!("{}", String::from_utf8(data.to_vec()).unwrap());
            infoJson = String::from_utf8(data.to_vec()).unwrap();
        } else {
            while let Some(chunk) = field.next().await {}
            //TODO: Save Files
        }
    }
    Ok(serde_json::from_str::<JudgeInformation>(&infoJson).unwrap())
}

#[web::post("/judge")]
async fn judge(mut payload: Multipart) -> Result<web::HttpResponse, web::Error> {
    //Read local file
    //let codes = fs::read_to_string("Test/Test.cpp").unwrap();

    //Ganerate uuid
    let id = Uuid::new_v4().to_string();
    let result = saveFile(payload, &id).await.unwrap();
    println!("{:?}", result);

    //Try to compile file with g++
    let compileResult = Command::new("g++")
        .arg("Test/Test.cpp")
        .arg("-o")
        .arg("Test/Test")
        .output()
        .unwrap();
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
    let mut output = Command::new("./Test/Test.exe")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let sec = Duration::from_secs(1);
    let (runSucceed, statusCode, timeoutTerminate) = match output.wait_timeout(sec).unwrap() {
        Some(status) => (status.success(), status.code().unwrap(), false),
        None => {
            output.kill().unwrap();
            (false, output.wait().unwrap().code().unwrap(), true)
        }
    };
    let duration = startTime.elapsed();
    if timeoutTerminate {
        let err = Err(JudgeError {
            errorType: JudgeErrorType::Runtime,
            error: format!("Time Limit Exceed"),
        });
        return err.map_err(|err| {
            web::error::ErrorBadRequest(serde_json::to_string(&err).unwrap()).into()
        });
    }
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
    output.stdout.unwrap().read_to_string(&mut results).unwrap();
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
