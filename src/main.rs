#[macro_use]
extern crate rocket;

use lazy_static::lazy_static;
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_okapi::rapidoc::*;
use rocket_okapi::{openapi, openapi_get_routes, settings::UrlObject};
use structsy::Ref;
// use rocket_okapi::swagger_ui::*;
use schemars::JsonSchema;
use std::str::FromStr;
use structopt::StructOpt;
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

lazy_static! {
    static ref DB: Structsy = {
        let db = Structsy::open("maps.persydb").expect("could not open database file");
        db.define::<Map>().unwrap();
        db
    };
    static ref CONFIG: Config = {
        let options = Options::from_args();
        Config {
            apikeys: std::fs::read_to_string(options.apikeys.clone().unwrap_or("./apikeys".into()))
                .unwrap_or_default()
                .lines()
                .map(ToString::to_string)
                .collect(),
            test_map_folder: options.test_map_folder.unwrap_or("./maps".into()),
            public_map_folder: options.public_map_folder.unwrap_or("./maps".into()),
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
    let query = db.query::<Map>().by_name(name);
    query.fetch().nth(0)
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
    match find_map(&db, &my_data.name) {
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

fn to_bad_request<T: ToString>(e: T) -> Status {
    eprintln!("{}", e.to_string());
    Status::BadRequest
}

fn to_internal_server_error<T: ToString>(e: T) -> Status {
    eprintln!("{}", e.to_string());
    Status::InternalServerError
}

#[openapi]
#[post("/create", format = "json", data = "<data>")]
async fn create_map(_key: ApiKey, data: Json<CreateMapData<'_>>) -> Result<(), Status> {
    let difficulty = Difficulty::from_str(data.difficulty).map_err(to_bad_request)?;
    let file = reqwest::get(data.url)
        .await
        .map_err(to_bad_request)?
        .bytes()
        .await
        .map_err(to_bad_request)?;

    let dir = CONFIG.test_map_folder.join("test");

    std::fs::create_dir_all(&dir).map_err(to_internal_server_error)?;

    std::fs::write(dir.join(data.name), file)
        .map_err(to_internal_server_error)?;

    add_or_update_map(&DB, data.name.to_string(), difficulty, State::New)
        .map_err(to_bad_request)?;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", openapi_get_routes![list_maps, create_map])
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
