#[macro_use]
extern crate rocket;

use lazy_static::lazy_static;
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_okapi::rapidoc::*;
use rocket_okapi::{openapi, openapi_get_routes, settings::UrlObject};
use schemars::JsonSchema;
use std::path::Path;
use std::str::FromStr;
use structopt::StructOpt;
use structsy::Ref;
use structsy::{Structsy, StructsyError, StructsyTx};
use structsy_derive::{queries, Persistent, PersistentEmbedded};
use strum::EnumString;

mod apikey;
mod common;
mod config;
mod options;

use apikey::ApiKey;
use config::Config;
use options::Options;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct CustomError {
    msg: String,
    code: u16,
}

type CustomStatus = (Status, Json<CustomError>);

lazy_static! {
    static ref DB: Structsy = {
        let db = Structsy::open("maps.persydb").expect("could not open database file");
        db.define::<Map>().unwrap();
        db
    };
    static ref CONFIG: Config = {
        let options = Options::from_args();
        Config {
            apikeys: std::fs::read_to_string(options.apikeys)
                .unwrap_or_default()
                .lines()
                .map(ToString::to_string)
                .collect(),
            test_map_folder: options.test_maps,
            public_map_folder: options.published_maps,
            dev: options.dev,
        }
    };
}

#[derive(
    Serialize,
    Deserialize,
    FromFormField,
    JsonSchema,
    PersistentEmbedded,
    Debug,
    EnumString,
    PartialEq,
    Clone,
    Copy,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Difficulty {
    Easy,
    Main,
    Hard,
    Insane,
}

#[derive(
    Serialize,
    Deserialize,
    FromFormField,
    JsonSchema,
    PersistentEmbedded,
    Debug,
    EnumString,
    PartialEq,
    Clone,
    Copy,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum State {
    New,
    Declined,
    Approved,
    Published,
}

#[derive(Serialize, Deserialize, JsonSchema, Persistent, Debug)]
struct Map {
    #[index]
    name: String,
    difficulty: Difficulty,
    state: State,
}

#[queries(Map)]
trait MapByName {
    fn by_name(self, name: &str) -> Self;
}

fn find_map(db: &Structsy, name: &str) -> Option<(Ref<Map>, Map)> {
    let query = db.query::<Map>().by_name(&name.to_lowercase());
    query.fetch().next()
}

fn add_or_update_map(
    db: &Structsy,
    name: String,
    difficulty: Difficulty,
    state: State,
) -> Result<(), StructsyError> {
    let my_data = Map {
        name,
        difficulty,
        state,
    };
    match find_map(db, &my_data.name) {
        None => {
            let mut tx = db.begin()?;
            tx.insert(&my_data)?;
            tx.commit()?;
        }
        Some((id, map)) => {
            let mut tx = db.begin()?;
            tx.update(&id, &Map { difficulty, ..map })?;
            tx.commit()?
        }
    }

    Ok(())
}

fn move_map<P: AsRef<Path>>(from: P, to: P) -> Result<(), std::io::Error> {
    let p = to.as_ref();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&from, to)?;
    std::fs::remove_file(from)
}

#[openapi]
#[get("/list?<name>&<state>&<difficulty>")]
fn list_maps(
    _key: ApiKey,
    name: Option<String>,
    state: Option<State>,
    difficulty: Option<Difficulty>,
) -> Json<Vec<Map>> {
    let query = DB.query::<Map>();

    let query = if let Some(name) = name {
        query.by_name(&name)
    } else {
        query
    };

    let values = query.into_iter().filter_map(|(_id, map)| {
        if let Some(state) = state {
            if map.state != state {
                return None;
            }
        };

        if let Some(difficulty) = difficulty {
            if map.difficulty != difficulty {
                return None;
            }
        };

        Some(map)
    });

    values.collect::<Vec<_>>().into()
}

#[derive(Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct CreateMapData<'r> {
    name: &'r str,
    difficulty: &'r str,
    url: &'r str,
}

#[derive(Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct ChangeMapDifficultyData<'r> {
    name: &'r str,
    difficulty: &'r str,
}

#[derive(Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct JustTheMapName<'r> {
    name: &'r str,
}

fn to_bad_request<T: ToString>(e: T) -> CustomStatus {
    eprintln!("{}", e.to_string());
    (
        Status::BadRequest,
        Json(CustomError {
            msg: "Something went wrong on client side!".to_string(),
            code: Status::BadRequest.code,
        }),
    )
}

fn to_custom_bad_request(msg: String) -> CustomStatus {
    eprintln!("{}", msg);
    (
        Status::BadRequest,
        Json(CustomError {
            msg,
            code: Status::BadRequest.code,
        }),
    )
}

fn to_internal_server_error<T: ToString>(e: T) -> CustomStatus {
    eprintln!("{}", e.to_string());
    (
        Status::InternalServerError,
        Json(CustomError {
            msg: "Something went wrong on server side!".to_string(),
            code: Status::InternalServerError.code,
        }),
    )
}

fn to_map_not_found_error<T: ToString>(e: T) -> CustomStatus {
    eprintln!("{}", e.to_string());
    (
        Status::NotFound,
        Json(CustomError {
            msg: "Map not found!".to_string(),
            code: Status::NotFound.code,
        }),
    )
}

#[openapi]
#[post("/recall", format = "json", data = "<data>")]
async fn recall_map(_key: ApiKey, data: Json<JustTheMapName<'_>>) -> Result<(), CustomStatus> {
    if let Some((id, map)) = find_map(&DB, data.name) {
        let mut tx = DB.begin().map_err(to_internal_server_error)?;
        tx.update(
            &id,
            &Map {
                state: State::New,
                ..map
            },
        )
        .map_err(to_internal_server_error)?;
        tx.commit().map_err(to_internal_server_error)?;
        Ok(())
    } else {
        Err(to_map_not_found_error(format!(
            "Map \"{}\" not found!",
            data.name
        )))
    }
}

#[openapi]
#[post("/decline", format = "json", data = "<data>")]
async fn decline_map(_key: ApiKey, data: Json<JustTheMapName<'_>>) -> Result<(), CustomStatus> {
    //TODO: Delete Map after 3Days from all Testservers
    if let Some((id, map)) = find_map(&DB, data.name) {
        if [State::Approved, State::New].contains(&map.state) {
            let mut tx = DB.begin().map_err(to_internal_server_error)?;
            tx.update(
                &id,
                &Map {
                    state: State::Declined,
                    ..map
                },
            )
            .map_err(to_internal_server_error)?;
            tx.commit().map_err(to_internal_server_error)?;
            Ok(())
        } else if map.state == State::Declined {
            Err(to_custom_bad_request(
                "This map is already declined!".to_string()
            ))
        } else {
            Err(to_custom_bad_request(format!(
                "Cannot go from state {:?} to {:?}!",
                map.state,
                State::Declined
            )))
        }
    } else {
        Err(to_map_not_found_error(format!(
            "Map \"{}\" not found!",
            data.name
        )))
    }
}

#[openapi]
#[post("/publish", format = "json", data = "<data>")]
async fn publish_map(_key: ApiKey, data: Json<JustTheMapName<'_>>) -> Result<(), CustomStatus> {
    if let Some((id, map)) = find_map(&DB, data.name) {
        if State::Approved == map.state {
            let mut tx = DB.begin().map_err(to_internal_server_error)?;
            tx.update(
                &id,
                &Map {
                    state: State::Published,
                    ..map
                },
            )
            .map_err(to_internal_server_error)?;
            tx.commit().map_err(to_internal_server_error)?;
            
            //TODO: Move Map to Public 
            let mut dirtemp = CONFIG.test_map_folder.join("test");
            let mut dirpublic = CONFIG.public_map_folder.join("");
            if Difficulty::Easy == map.difficulty {
                dirpublic.join("easy");
            }else if Difficulty::Hard == map.difficulty{
                dirpublic.join("hard");
            }else if Difficulty::Insane == map.difficulty{
                dirpublic.join("insane");
            }else if Difficulty::Main == map.difficulty{
                dirpublic.join("main");
            } //TODO else with an error message that difficulty is invalid.
            
            //TODO Move map to Official map folder
            //let mut tempfile = dirtemp + map.name + ".map";   // String not a pathstring .join(map.name + ".map");
            //let mut publicfile = dirtemp + map.name + ".map"; // String not a pathstring .join(map.name + ".map");
            //move_map(tempfile, publicfile);
            //TODO commit to Github and trigger Slaves
            Ok(())
        } else if State::Published == map.state {
            Err(to_custom_bad_request(
                "This map is already published!".to_string(),
            ))
        } else {
            Err(to_custom_bad_request(format!(
                "Cannot go from state {:?} to {:?}!",
                map.state,
                State::Published
            )))
        }
    } else {
        Err(to_map_not_found_error(format!(
            "Map \"{}\" not found!",
            data.name
        )))
    }
}

#[openapi]
#[post("/approve", format = "json", data = "<data>")]
async fn approve_map(_key: ApiKey, data: Json<JustTheMapName<'_>>) -> Result<(), CustomStatus> {
    if let Some((id, map)) = find_map(&DB, data.name) {
        if [State::Declined, State::New].contains(&map.state) {
            let mut tx = DB.begin().map_err(to_internal_server_error)?;
            tx.update(
                &id,
                &Map {
                    state: State::Approved,
                    ..map
                },
            )
            .map_err(to_internal_server_error)?;
            tx.commit().map_err(to_internal_server_error)?;
            Ok(())
        } else if map.state == State::Approved {
            Err(to_custom_bad_request(
                "This map is already Approved!".to_string()
            ))
        } else {
            Err(to_custom_bad_request(format!(
                "Cannot go from state {:?} to {:?}!",
                map.state,
                State::Approved
            )))
        }
    } else {
        Err(to_map_not_found_error(format!(
            "Map \"{}\" not found!",
            data.name
        )))
    }
}

#[openapi]
#[post("/change_difficulty", format = "json", data = "<data>")]
async fn change_map_difficulty(
    _key: ApiKey,
    data: Json<ChangeMapDifficultyData<'_>>,
) -> Result<(), CustomStatus> {
    let difficulty = Difficulty::from_str(data.difficulty).map_err(to_bad_request)?;
    
    if let Some((id, map)) = find_map(&DB, &data.name.to_lowercase()) {
        let mut tx = DB.begin().map_err(to_internal_server_error)?;
        tx.update(&id, &Map { difficulty, ..map })
            .map_err(to_internal_server_error)?;
        tx.commit().map_err(to_internal_server_error)?;
        Ok(())
    } else {
        Err(to_map_not_found_error(format!(
            "Map \"{}\" not found!",
            data.name
        )))
    }
}

#[openapi]
#[post("/create", format = "json", data = "<data>")]
async fn create_map(_key: ApiKey, data: Json<CreateMapData<'_>>) -> Result<(), CustomStatus> {
    let difficulty = Difficulty::from_str(data.difficulty).map_err(to_bad_request)?;
    let file = reqwest::get(data.url)
        .await
        .map_err(to_bad_request)?
        .bytes()
        .await
        .map_err(to_bad_request)?;

    let dir = CONFIG.test_map_folder.join("test");

    std::fs::create_dir_all(&dir).map_err(to_internal_server_error)?;

    let mut name = data.name.to_lowercase();

    if name.ends_with(".map") {
        name = name[0..name.len() - 4].to_string()
    }

    std::fs::write(dir.join(&format!("{}.map", name)), file).map_err(to_internal_server_error)?;

    add_or_update_map(&DB, name, difficulty, State::New).map_err(to_bad_request)?;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    // this is needed in order to display help texts, because they dont work in lazy_static
    let _ = Options::from_args();
    rocket::build()
        .mount(
            "/",
            openapi_get_routes![
                list_maps,
                create_map,
                change_map_difficulty,
                approve_map,
                publish_map,
                recall_map,
                decline_map
            ],
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                ui: UiConfig {
                    theme: Theme::Dark,
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .register("/", catchers![common::bad_request, common::unauthorized])
}
