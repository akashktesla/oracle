#![allow(warnings)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MConstraint {
    Fixed(f32),
    Ratio(usize),
}

impl MConstraint {
    pub fn label(&self) -> &'static str {
        match self {
            MConstraint::Fixed(_) => "Fixed",
            MConstraint::Ratio(_) => "Ratio",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecipeIngredient {
    pub name: String,
    pub constraint: MConstraint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recipe {
    pub name: String,
    pub target_calories: f32,
    pub ingredients: Vec<RecipeIngredient>,
}

impl Recipe {
    pub fn new(name: impl Into<String>, target_calories: f32) -> Self {
        Recipe {
            name: name.into(),
            target_calories,
            ingredients: Vec::new(),
        }
    }
}

pub type SolvedRecipe = HashMap<String, f32>;

#[derive(Debug, Clone, PartialEq)]
pub enum SolveError {
    UnknownIngredient(String),
    NoCaloriesRemaining,
    EmptyRecipe,
}

impl std::fmt::Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolveError::UnknownIngredient(name) => {
                write!(f, "unknown ingredient: {name} (not in ingredient database)")
            }
            SolveError::NoCaloriesRemaining => {
                write!(f, "no calories remain to distribute across ratio ingredients")
            }
            SolveError::EmptyRecipe => write!(f, "recipe has no ingredients"),
        }
    }
}

pub fn default_ingredients_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        let candidate = PathBuf::from(home).join("projects/oracle/datasets/ingredients.csv");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("datasets/ingredients.csv")
}

pub fn recipes_store_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".local/share/oracle/recipes.json")
    } else {
        PathBuf::from("recipes.json")
    }
}

pub fn load_ingredient_db(path: &std::path::Path) -> Result<HashMap<String, f32>, String> {
    let mut rdr = csv::Reader::from_path(path)
        .map_err(|e| format!("couldn't open ingredient database at {}: {e}", path.display()))?;
    let mut map = HashMap::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("bad row in ingredient database: {e}"))?;
        if record.len() < 2 {
            continue;
        }
        let name = record[0].trim().to_string();
        if name.is_empty() {
            continue;
        }
        let cpg: f32 = record[1].trim().parse().unwrap_or(0.0);
        map.insert(name, cpg);
    }
    Ok(map)
}

pub fn solve_recipe(
    recipe: &Recipe,
    ingredients_db: &HashMap<String, f32>,
) -> Result<SolvedRecipe, SolveError> {
    if recipe.ingredients.is_empty() {
        return Err(SolveError::EmptyRecipe);
    }

    let mut total_fixed_calories = 0.0f32;
    let mut total_ratio_weight: usize = 0;
    let mut ratio_items: Vec<&RecipeIngredient> = Vec::new();
    let mut result: SolvedRecipe = HashMap::new();

    for item in &recipe.ingredients {
        let cpg = ingredients_db
            .get(&item.name)
            .copied()
            .ok_or_else(|| SolveError::UnknownIngredient(item.name.clone()))?;

        match item.constraint {
            MConstraint::Fixed(grams) => {
                total_fixed_calories += grams * cpg;
                result.insert(item.name.clone(), grams);
            }
            MConstraint::Ratio(weight) => {
                total_ratio_weight += weight;
                ratio_items.push(item);
            }
        }
    }

    if ratio_items.is_empty() {
        return Ok(result);
    }

    let remaining_calories = recipe.target_calories - total_fixed_calories;
    if remaining_calories <= 0.0 || total_ratio_weight == 0 {
        return Err(SolveError::NoCaloriesRemaining);
    }

    for item in ratio_items {
        let MConstraint::Ratio(weight) = item.constraint else {
            continue;
        };
        let cpg = ingredients_db[&item.name];
        let share_calories = (weight as f32 / total_ratio_weight as f32) * remaining_calories;
        let grams = share_calories / cpg;
        result.insert(item.name.clone(), grams);
    }

    Ok(result)
}

pub fn load_recipes() -> Result<Vec<Recipe>, String> {
    let path = recipes_store_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("couldn't read {}: {e}", path.display()))?;
    if data.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&data).map_err(|e| format!("couldn't parse {}: {e}", path.display()))
}

pub fn save_recipes(recipes: &[Recipe]) -> Result<(), String> {
    let path = recipes_store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("couldn't create {}: {e}", parent.display()))?;
    }
    let data = serde_json::to_string_pretty(recipes)
        .map_err(|e| format!("couldn't serialize recipes: {e}"))?;
    fs::write(&path, data).map_err(|e| format!("couldn't write {}: {e}", path.display()))
}

pub fn upsert_recipe(recipe: Recipe) -> Result<(), String> {
    let mut recipes = load_recipes()?;
    if let Some(existing) = recipes.iter_mut().find(|r| r.name == recipe.name) {
        *existing = recipe;
    } else {
        recipes.push(recipe);
    }
    save_recipes(&recipes)
}

pub fn delete_recipe(name: &str) -> Result<bool, String> {
    let mut recipes = load_recipes()?;
    let before = recipes.len();
    recipes.retain(|r| r.name != name);
    let removed = recipes.len() != before;
    if removed {
        save_recipes(&recipes)?;
    }
    Ok(removed)
}

