use rocket::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::path::Path;
use structsy_derive::{Persistent, PersistentEmbedded};
use strum::EnumString;

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
pub enum Difficulty {
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
pub enum State {
    New,
    Declined,
    Approved,
    Published,
}

#[derive(Serialize, Deserialize, JsonSchema, Persistent, Debug)]
pub struct Map {
    #[index]
    pub name: String,
    pub difficulty: Difficulty,
    pub state: State,
    pub created_at: u64,
    pub last_changed: u64,
}
