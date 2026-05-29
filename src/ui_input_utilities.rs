use std::{collections::BTreeSet};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub enum GridType {
    Simple,
    Scandi
}

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub enum Difficulty {
    Beginner,
    Medium,
    Expert
}

#[derive(Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
#[wasm_bindgen]
pub enum Theme {
    ArtsAndCulture,
    NatureAndScience,
    SportsAndGames,
    FoodAndDrink,
    HistoryAndSociety,
    Technology,
    Seasonal,
    Wordplay,
    Random
}

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct UserSettings {
    grid_type: GridType,
    difficulty_level: Difficulty,
    themes: BTreeSet<Theme>
}

impl UserSettings {
    pub fn get_grid_type(&self) -> &GridType {
        return &self.grid_type;
    }

    pub fn get_difficulty_level(&self) -> &Difficulty {
        return &self.difficulty_level;
    }
}




