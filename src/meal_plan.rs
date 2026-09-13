#![allow(warnings)]
use csv::Reader;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone)]
enum Constraint {
    Fixed(f32),
    Ratio(usize),
}

fn load_map() -> HashMap<String, f32> {
    let path = "/home/akash/projects/oracle/datasets/ingredients.csv";
    let mut rdr = Reader::from_path(path).unwrap();
    let mut map: HashMap<String, f32> = HashMap::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let name = record[0].trim().to_string();
        let cpg: f32 = record[1].trim().parse().unwrap_or(0.);
        map.insert(name, cpg);
    }
    println!("map: {map:?}");
    return map;
}

pub fn main() {
    let ingredients = load_map();
    let imap = HashMap::from([
        ("rice".to_string(), Constraint::Ratio(1)),
        ("chicken".to_string(), Constraint::Fixed(250.)),
        ("ghee".to_string(), Constraint::Fixed(2.5)),
        ("tomato".to_string(), Constraint::Fixed(70.)),
        ("onion".to_string(), Constraint::Fixed(70.)),
        ("mint".to_string(), Constraint::Fixed(50.)),
    ]);
    let target_calories = 1100.;
    let mut total_calories = 0.;
    let mut total_ratios = 0;
    let mut to_solve = Vec::new();
    for i in imap.keys() {
        // let ingrt = i.to_string()
        println!("i: {i:?}");
        let constraint = imap[i].clone();
        if let Constraint::Fixed(val) = imap[i] {
            let cpg = ingredients[i];
            let cals = val * cpg;
            total_calories += cals;
            println!("calories: {cals:?}");
        } else if let Constraint::Ratio(ratio) = imap[i] {
            total_ratios += ratio;
            to_solve.push(i.clone());
        }
    }
    println!("total_calories: {total_calories:?}");
    println!("to_solve: {to_solve:?}");
    let remaining_cals = target_calories - total_calories;
    for i in &to_solve {
        let Constraint::Ratio(ratio) = imap[i] else {
            continue;
        };
        let cpg = ingredients[i];
        let amount = ((ratio as f32 / total_ratios as f32) * remaining_cals as f32) / cpg;
        println!("{i}: {amount:?}");
    }
}
