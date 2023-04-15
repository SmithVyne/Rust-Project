use uuid::Uuid;
use actix_files as fs;
use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::{get, post, web, Error, App, HttpRequest, HttpResponse, HttpServer, Responder, http::header};
use std::sync::Mutex;
use sqlite::State;


extern crate tera;
use tera::{Result, Context, Tera};

use serde::Serialize;

#[macro_use]
extern crate lazy_static;



struct AppStateWithCounter {
    aX: Mutex<i32>,
    aY: Mutex<i32>,
    aZ: Mutex<i32>,
}

async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut aX = data.aX.lock().unwrap(); 
    let mut aY = data.aY.lock().unwrap(); 
    let mut aZ = data.aZ.lock().unwrap(); 

    format!("Pos now: {aX}-{aY}-{aZ}") 

}

async fn addX(data: web::Data<AppStateWithCounter>) -> String {
    let mut aX = data.aX.lock().unwrap(); 
    let mut aY = data.aY.lock().unwrap(); 
    let mut aZ = data.aZ.lock().unwrap(); 

    *aX += 10;

    format!("Pos now: {aX}-{aY}-{aZ}") 
}

#[get("/check")] 
async fn check(date: web::Header<header::Date>) -> String {
    println!("-{}-", date.to_string());
    format!("Request was sent at {}", date.to_string())
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

async fn handler(req: HttpRequest, data: web::Data<AppStateWithCounter>) -> impl Responder {
    let key = getKey(&req);
    let axis = getAxis(&req);
    let value = getValue(&req);

    let mut aX = data.aX.lock().unwrap(); 
    let mut aY = data.aY.lock().unwrap(); 
    let mut aZ = data.aZ.lock().unwrap(); 

    // println!("{}", key.is_some().to_string());
    // println!("{}", axis.is_some().to_string());
    // println!("{}", value.is_some().to_string());

    if key.is_some() || axis.is_some() || value.is_some() {
        if key.unwrap() == "10sg323Pt4s353sd353G" {

            let intValue = value.unwrap().parse::<i32>().unwrap();

            if axis.unwrap() == "1" {
                *aX += intValue;
            }
            if axis.unwrap() == "2" {
                *aY += intValue;
            }
            if axis.unwrap() == "3" {
                *aZ += intValue;
            }

            let connection = sqlite::open("resps.db").unwrap();
            let query = format!("INSERT INTO resps (axis, value) VALUES ({}, {});", axis.unwrap(), value.unwrap());
            connection.execute(query).unwrap();

            return format!("Right key | axis: {}; | value: {};", axis.unwrap(), value.unwrap())
        } else {
            return "Access denied".to_owned();
        }
    }
    return format!("Pos now: {aX}-{aY}-{aZ}") 
}

fn getKey<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    req.headers().get("key")?.to_str().ok()
}

fn getAxis<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    req.headers().get("axis")?.to_str().ok()
}

fn getValue<'a>(req: &'a HttpRequest) -> Option<&'a str> {
    req.headers().get("value")?.to_str().ok()
}

async fn template(req: HttpRequest, data: web::Data<AppStateWithCounter>) -> impl Responder {

    let mut aX = data.aX.lock().unwrap(); 
    let mut aY = data.aY.lock().unwrap(); 
    let mut aZ = data.aZ.lock().unwrap(); 

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("title", "template");
    context.insert("message", "Hello world");
    context.insert("aX", &format!("{aX}"));
    context.insert("aY", &format!("{aY}"));
    context.insert("aZ", &format!("{aZ}"));
    let rendered = tera.render("index.html", &context).unwrap();
    HttpResponse::Ok().body(rendered)
}

// #[get("/{static:.*}")]
// async fn useJs(req: HttpRequest) -> actix_web::Result<fs::NamedFile, Error> {
//     let path: std::path::PathBuf = req.match_info().query("static").parse().unwrap();
//     let file = fs::NamedFile::open(path)?;
//     Ok(file
//         .use_last_modified(true)
//         .set_content_disposition(ContentDisposition {
//             disposition: DispositionType::Attachment,
//             parameters: vec![],
//         }))
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let aXaYaZ = web::Data::new(AppStateWithCounter {
        aX: Mutex::new(0),
        aY: Mutex::new(0),
        aZ: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
            // .service(useJs)
            .service(fs::Files::new("/static", ".").show_files_listing())
            .app_data(aXaYaZ.clone())
            .service(check)
            // .service(fs::Files::new("/reporting", "./static").index_file("index.html"))
            .route("/add", web::get().to(addX))
            .route("/handler", web::to(handler))
            .route("/", web::get().to(index))
            .route("/template", web::get().to(template))
            
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
