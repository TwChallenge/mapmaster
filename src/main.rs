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
use std::time::SystemTime;
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

fn map_to_test_vote_string(map: &Map) -> String {
    let approved = match map.state {
        State::Approved => "☑",
        State::Declined => "☒",
        _ => "🆕",
    };
    let difficulty = format!("[{}]", map.difficulty);
    let difficulty = match map.difficulty {
        Difficulty::Insane => format!("{: <8}", difficulty),
        _ => format!("{: <9}", difficulty),
    };
    format!(
        "add_vote \"{} {} {}\" \"change_map \\\"{}\\\"\"",
        approved, difficulty, map.name, map.name
    )
}

fn map_to_vote_string(map: &Map) -> String {
    format!(
        "add_vote \"{}\" \"change_map \\\"{}/{}\\\"\"",
        map.name,
        map.difficulty.to_string().to_lowercase(),
        map.name
    )
}

fn generate_test_votes(maps: &[Map]) -> String {
    maps.iter()
        .map(map_to_test_vote_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_published_votes(maps: &[Map]) -> String {
    let new = maps.iter().take(6);
    let mut other = maps.iter().skip(6).collect::<Vec<_>>();
    other.sort_by_key(|m| &m.name);
    let mut text = vec!["add_vote \"─── NEW MAPS ───\" \"info\"".to_string()];
    text.extend(new.map(map_to_vote_string));
    text.push("add_vote \"────────────────\" \"info\"".to_string());
    text.extend(other.into_iter().map(map_to_vote_string));
    text.join("\n")
}

fn update_votes(db: &Structsy) -> Result<(), CustomStatus> {
    let query = db.query::<Map>().fetch();
    let mut test = Vec::new();
    let mut easy = Vec::new();
    let mut main = Vec::new();
    let mut hard = Vec::new();
    let mut insane = Vec::new();
    for map in query.map(|(_id, map)| map) {
        if [State::New, State::Approved, State::Declined].contains(&map.state) {
            test.push(map);
        } else if map.state == State::Published {
            use Difficulty::*;
            match map.difficulty {
                Easy => easy.push(map),
                Main => main.push(map),
                Hard => hard.push(map),
                Insane => insane.push(map),
            }
        }
    }

    test.sort_by_key(Map::created_at);
    easy.sort_by_key(Map::created_at);
    main.sort_by_key(Map::created_at);
    hard.sort_by_key(Map::created_at);
    insane.sort_by_key(Map::created_at);

    std::fs::create_dir_all(&CONFIG.test_map_folder).map_err(to_internal_server_error)?;

    let base = &CONFIG.public_map_folder;
    let easy_folder = base.join(Difficulty::Easy);
    let main_folder = base.join(Difficulty::Main);
    let hard_folder = base.join(Difficulty::Hard);
    let insane_folder = base.join(Difficulty::Insane);

    std::fs::create_dir_all(&easy_folder).map_err(to_internal_server_error)?;
    std::fs::create_dir_all(&main_folder).map_err(to_internal_server_error)?;
    std::fs::create_dir_all(&hard_folder).map_err(to_internal_server_error)?;
    std::fs::create_dir_all(&insane_folder).map_err(to_internal_server_error)?;

    std::fs::write(
        CONFIG.test_map_folder.join("votes.cfg"),
        generate_test_votes(&test),
    )
    .map_err(to_internal_server_error)?;
    std::fs::write(
        easy_folder.join("votes.cfg"),
        generate_published_votes(&easy),
    )
    .map_err(to_internal_server_error)?;
    std::fs::write(
        main_folder.join("votes.cfg"),
        generate_published_votes(&main),
    )
    .map_err(to_internal_server_error)?;
    std::fs::write(
        hard_folder.join("votes.cfg"),
        generate_published_votes(&hard),
    )
    .map_err(to_internal_server_error)?;
    std::fs::write(
        insane_folder.join("votes.cfg"),
        generate_published_votes(&insane),
    )
    .map_err(to_internal_server_error)?;

    Ok(())
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

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Difficulty::*;
        match self {
            Easy => write!(f, "Easy"),
            Main => write!(f, "Main"),
            Hard => write!(f, "Hard"),
            Insane => write!(f, "Insane"),
        }
    }
}

impl AsRef<Path> for Difficulty {
    fn as_ref(&self) -> &Path {
        use Difficulty::*;
        let s = match self {
            Easy => "easy",
            Main => "main",
            Hard => "hard",
            Insane => "insane",
        };
        Path::new(s)
    }
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
    created_at: u64,
    last_changed: u64,
}

impl Map {
    fn created_at(&self) -> u64 {
        self.created_at
    }
}

#[queries(Map)]
trait MapByName {
    fn by_name(self, name: &str) -> Self;
    fn by_state(self, #[allow(unused)] state: &State) -> Self;
    fn by_difficulty(self, #[allow(unused)] difficulty: &Difficulty) -> Self;
}

#[queries(Map)]
trait MapByState {}

fn find_map(db: &Structsy, name: &str) -> Option<(Ref<Map>, Map)> {
    let query = db.query::<Map>().by_name(&name.to_lowercase());
    query.fetch().next()
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

fn get_current_time() -> Result<u64, Either<StructsyError, Box<dyn std::error::Error>>> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| Either::Right(e.into()))?
        .as_secs())
}

fn add_or_update_map(
    db: &Structsy,
    name: String,
    difficulty: Difficulty,
    state: State,
) -> Result<(), Either<StructsyError, Box<dyn std::error::Error>>> {
    let now = get_current_time()?;
    let my_data = Map {
        name: name.to_lowercase(),
        difficulty,
        state,
        created_at: now,
        last_changed: now,
    };
    match find_map(db, &my_data.name) {
        None => {
            let mut tx = db.begin().map_err(Either::Left)?;
            tx.insert(&my_data).map_err(Either::Left)?;
            tx.commit().map_err(Either::Left)?;
        }
        Some((id, map)) => {
            let mut tx = db.begin().map_err(Either::Left)?;
            tx.update(
                &id,
                &Map {
                    difficulty,
                    last_changed: now,
                    ..map
                },
            )
            .map_err(Either::Left)?;
            tx.commit().map_err(Either::Left)?
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
        let map_name = format!("{}.map", map.name);
        tx.update(
            &id,
            &Map {
                state: State::New,
                last_changed: get_current_time().map_err(either_to_custom_status)?,
                ..map
            },
        )
        .map_err(to_internal_server_error)?;

        if map.state == State::Published {
            let source_dir = CONFIG.public_map_folder.join(map.difficulty);
            let target_dir = &CONFIG.test_map_folder;

            std::fs::create_dir_all(&target_dir).map_err(to_internal_server_error)?;
            move_map(source_dir.join(&map_name), target_dir.join(&map_name))
                .map_err(to_internal_server_error)?;
        }
        tx.commit().map_err(to_internal_server_error)?;
        update_votes(&DB)?;
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
    if let Some((id, map)) = find_map(&DB, &data.name.to_lowercase()) {
        if [State::Approved, State::New].contains(&map.state) {
            let mut tx = DB.begin().map_err(to_internal_server_error)?;
            tx.update(
                &id,
                &Map {
                    state: State::Declined,
                    last_changed: get_current_time().map_err(either_to_custom_status)?,
                    ..map
                },
            )
            .map_err(to_internal_server_error)?;
            tx.commit().map_err(to_internal_server_error)?;
            update_votes(&DB)?;
            Ok(())
        } else if map.state == State::Declined {
            Err(to_custom_bad_request(
                "This map is already declined!".to_string(),
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
            let map_name = format!("{}.map", map.name);
            tx.update(
                &id,
                &Map {
                    state: State::Published,
                    last_changed: get_current_time().map_err(either_to_custom_status)?,
                    ..map
                },
            )
            .map_err(to_internal_server_error)?;

            let source_dir = &CONFIG.test_map_folder;
            let target_dir = CONFIG.public_map_folder.join(map.difficulty);

            std::fs::create_dir_all(&target_dir).map_err(to_internal_server_error)?;
            move_map(source_dir.join(&map_name), target_dir.join(&map_name))
                .map_err(to_internal_server_error)?;
            tx.commit().map_err(to_internal_server_error)?;
            update_votes(&DB)?;
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
                    last_changed: get_current_time().map_err(either_to_custom_status)?,
                    ..map
                },
            )
            .map_err(to_internal_server_error)?;
            tx.commit().map_err(to_internal_server_error)?;
            update_votes(&DB)?;
            Ok(())
        } else if map.state == State::Approved {
            Err(to_custom_bad_request(
                "This map is already Approved!".to_string(),
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

    if let Some((id, map)) = find_map(&DB, data.name) {
        let mut tx = DB.begin().map_err(to_internal_server_error)?;

        tx.update(
            &id,
            &Map {
                difficulty,
                last_changed: get_current_time().map_err(either_to_custom_status)?,
                ..map
            },
        )
        .map_err(to_internal_server_error)?;
        tx.commit().map_err(to_internal_server_error)?;
        update_votes(&DB)?;
        Ok(())
    } else {
        Err(to_map_not_found_error(format!(
            "Map \"{}\" not found!",
            data.name
        )))
    }
}

fn either_to_custom_status(
    either: Either<StructsyError, Box<dyn std::error::Error>>,
) -> CustomStatus {
    use Either::*;
    match either {
        Left(l) => to_bad_request(l),
        Right(r) => to_internal_server_error(r),
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

    let dir = &CONFIG.test_map_folder;

    std::fs::create_dir_all(&dir).map_err(to_internal_server_error)?;

    let name = data.name.to_lowercase();

    let name = if name.ends_with(".map") {
        name[0..name.len() - 4].to_string()
    } else {
        name
    };

    std::fs::write(dir.join(&format!("{}.map", name)), file).map_err(to_internal_server_error)?;

    let res = add_or_update_map(&DB, name, difficulty, State::New).map_err(either_to_custom_status);

    update_votes(&DB)?;

    res
}

#[launch]
fn rocket() -> _ {
    // this is needed in order to display help texts, because they dont work in lazy_static
    let _ = Options::from_args();

    let _ = update_votes(&DB);

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
