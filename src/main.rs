use std::{io::Write, io, fs};
//use std::collections::HashMap;
use mini_markdown::render;
use actix_files::Files;
use actix_web::{web::Data, web, get, App, HttpServer, HttpResponse, Responder};
use handlebars::Handlebars;
use serde::{Serialize, Deserialize};
use serde_json::json;
use string_join::display::Join;

#[get("/")]
async fn index(hb: Data<Handlebars<'_>>) -> impl Responder {
    let note_paths = listdir("notes".to_string(), false);
    let note_paths_full = listdir("notes".to_string(), true);

    let mut newnote_paths = note_paths.clone();
    // add number of notes in each folder
    for i in 0..note_paths_full.len() {
        let individual_paths = listdir(note_paths_full[i].to_string(), false);
        let numnotes = individual_paths.len().to_string();
        let newname = "".join(&[&note_paths[i], " (", &numnotes, ")"]);
        newnote_paths[i] = newname;
    }
    
    let mut notes_vec: Vec<[String; 2]> = vec![];
    for i in 0..note_paths.len() {
        let cont: [String; 2] = [note_paths[i].clone(), newnote_paths[i].clone()];
        notes_vec.push(cont);
    }

    let data = json!({
        "notes": notes_vec,
    });

    let html = hb.render("index", &data).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

#[derive(Serialize, Deserialize)]
pub struct MyParams {
    foldername: String,
    title: String,
    data: String,
}

// create note (POST)
async fn handle_post(hb: Data<Handlebars<'_>>, params: web::Form<MyParams>)  -> impl Responder {
    // get all folders
    let folder_paths = listdir("notes".to_string(), false); //test, fun, school
    let full_folder_path = "".join(&["./notes/", &params.foldername]);

    // if path doesn't exist, create dir
    if ! folder_paths.contains(&params.foldername) {
        fs::create_dir_all(&full_folder_path).expect("Error creating dir.");
    }

    // get all notes
    let existing_note_paths = listdir(full_folder_path, false); //a.md, b.md
    let fulltitle: String = if params.title.ends_with(".md") {
        params.title.clone()
    } else {
        "".join(&[&params.title, ".md"])
    };
    
    let fullnotepath: String = "".join(&["./notes/", &params.foldername, "/", &fulltitle]);        

    // check if note exists
    if ! existing_note_paths.contains(&fulltitle) {
        //create note
        let mut file = fs::File::create(fullnotepath).expect("Error creating file");
        file.write_all(params.data.as_bytes()).expect("Error writing to file");
    } 

    let data = json!({
        "notes_folder": params.foldername,
        "title": params.title,
    });

    let html = hb.render("newnote", &data).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

fn listdir(x: String, fullpath: bool) -> Vec<String> {
    let paths = fs::read_dir(x).unwrap();
    let mut split_vec: Vec<String> = vec![];
    for path in paths {
        let path_string = path.unwrap().path().display().to_string();
        let vec: Vec<&str> = path_string.split("/").collect();
        
        if fullpath {
            if ! vec[vec.len()-1].starts_with(".") { //skip dotfiles (.DS_Store)
                split_vec.push(path_string);
            }
        } else {
            if ! vec[vec.len()-1].starts_with(".") {
                split_vec.push(vec[vec.len()-1].to_string());
            }
        }
    }
    split_vec
}

// for debugging
/* fn print_variable_type<K>(_: &K) {
    println!("{}", std::any::type_name::<K>())
} */

//edit notes
async fn handle_edit(hb: Data<Handlebars<'_>>, params: web::Form<MyParams>) -> impl Responder {
    let fullnotepath = "".join(&["./notes/", &params.foldername, "/", &params.title]);

    // rewrite file with new data
    let mut file = fs::File::create(fullnotepath).expect("Error opening file");
    file.write(params.data.as_bytes()).expect("Error writing to file");

    let data = json!({
        "notes_folder": params.foldername,
        "title": params.title,
    });

    let html = hb.render("newnote", &data).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
} 

#[derive(Serialize, Deserialize)]
pub struct DelParams {
    foldername: String,
    title: String,
}
async fn handle_delete(hb: Data<Handlebars<'_>>, params: web::Form<DelParams>) -> impl Responder {
    let fullnotepath = "".join(&["./notes/", &params.foldername, "/", &params.title]);

    //println!("deleting {}", fullnotepath);
    fs::remove_file(fullnotepath).expect("Error deleting file");

    let data = json!({
        "notes_folder": params.foldername,
        "title": params.title,
    });

    let html = hb.render("newnote", &data).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

#[derive(Serialize, Deserialize)]
pub struct FolderDelParams {
    foldername: String,
}
async fn handle_deletefolder(hb: Data<Handlebars<'_>>, params: web::Form<FolderDelParams>) -> impl Responder {
    let fullnotepath = "".join(&["./notes/", &params.foldername]);

    fs::remove_dir_all(fullnotepath).expect("Error deleting folder");

    let data = json!({
        "notes_folder": "../",
        "title": "",
    });

    let html = hb.render("newnote", &data).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

async fn folders(path: web::Path<String>, hb: Data<Handlebars<'_>>) -> impl Responder {
    // add ./notes/ prefix
    let fullpath = "".join(&["./notes/", &path.to_string()]);

    let note_paths = listdir(fullpath.clone(), false);
    let note_paths_full = listdir(fullpath.clone(), true);
    let mut contents: Vec<String> = vec![];
    let mut og_contents: Vec<String> = vec![];
    for i in &note_paths_full {
        let res = fs::read_to_string(i.to_string()).expect("Error reading file.");
        
        og_contents.push(res.clone());
        let html_output = render(res.as_str());
        contents.push(html_output);
    }

    // make vec containing notes [path, content]
    let mut all_notes_vec: Vec<[String; 3]> = vec![];
    for i in 0..note_paths.len() {
        let cont: [String; 3] = [note_paths[i].clone(), contents[i].clone(), og_contents[i].clone()];
        all_notes_vec.push(cont);
    }
    
    let data = json!({
        "notes_folder": &path.to_string(),
        "notes": note_paths,
        "note_contents": all_notes_vec,
    });

    let html = hb.render("notes", &data).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    //let address = "https://rust-notes.onrender.com".to_string();
    // comment the line above and uncomment the line below to run locally
    let address = "0.0.0.0:8080".to_string();

    let template_service = {
        let mut handlebars = Handlebars::new();

        handlebars
            .register_templates_directory(".html", "./templates")
            .unwrap();

        Data::new(handlebars)
    };

    let server = move || App::new()
        .app_data(template_service.clone())
        .service(Files::new("/public", "web/public").show_files_listing())
        .service(index)
        .service(web::resource("/post").route(web::post().to(handle_post)))
        .service(web::resource("/edit").route(web::post().to(handle_edit)))
        .service(web::resource("/delete").route(web::post().to(handle_delete)))
        .service(web::resource("/deletefolder").route(web::post().to(handle_deletefolder)))
        .route("/notes/{folder}", web::get().to(folders));

    HttpServer::new(server)
        .bind(address)?
        .run()
        .await
}