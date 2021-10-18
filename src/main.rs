#[macro_use]
extern crate rocket;

use lazy_static::lazy_static;
use rocket::response::status::BadRequest;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket_okapi::rapidoc::*;
use rocket_okapi::{openapi, openapi_get_routes, settings::UrlObject};
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
            base: options.base.unwrap_or("./maps".into()),
            dev: options.dev,
        }
    };
}

#[derive(Serialize, Deserialize, JsonSchema, PersistentEmbedded, Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Difficulty {
    Easy,
    Main,
    Hard,
    Insane,
}

#[derive(Serialize, Deserialize, JsonSchema, PersistentEmbedded, Debug, EnumString)]
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

// We define a trait with our own logic and we ask structsy to generate the query on the MyData struct
#[queries(Map)]
trait MapQuery {
    // here is our condition method, to notice that the name of the parameter has to be exactly the same of the struct field.
    fn by_name(self, name: &str) -> Self;
}

fn add_map(
    db: &Structsy,
    name: String,
    difficulty: Difficulty,
    state: State,
) -> Result<(), StructsyError> {
    let query = db.query::<Map>().by_name(&name);
    if query.fetch().count() == 0 {
        let my_data = Map {
            name,
            difficulty,
            state,
        };
        let mut tx = db.begin()?;
        tx.insert(&my_data)?;
        tx.commit()?;
    }

    Ok(())
}

#[openapi]
#[get("/list?<name>&<test>")]
fn list_maps(_key: ApiKey, name: Option<String>, test: Option<bool>) -> Json<Vec<Map>> {
    DB.scan::<Map>()
        .map(|it| it.map(|m| m.1).collect::<Vec<_>>())
        .unwrap_or_default()
        .into()
}

#[derive(Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct CreateMapData<'r> {
    name: &'r str,
    difficulty: &'r str,
    url: &'r str,
}

fn to_bad_request<T: ToString>(e: T) -> BadRequest<String> {
    eprintln!("{}", e.to_string());
    BadRequest(Some(format!(
        "Something went wrong and I'm pretty sure it isn't our fault. Fix your request, mate!"
    )))
}

#[openapi]
#[post("/create", format = "json", data = "<data>")]
fn create_maps(_key: ApiKey, data: Json<CreateMapData<'_>>) -> Result<(), BadRequest<String>> {
    add_map(
        &DB,
        data.name.to_string(),
        Difficulty::from_str(data.difficulty).map_err(to_bad_request)?,
        State::New,
    )
    .map_err(to_bad_request)?;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", openapi_get_routes![list_maps, create_maps])
        // .mount(
        //     "/swagger-ui/",
        //     make_swagger_ui(&SwaggerUIConfig {
        //         url: "../openapi.json".to_owned(),
        //         ..Default::default()
        //     }),
        // )
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
