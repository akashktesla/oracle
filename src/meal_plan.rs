#![allow(warnings)]
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone)]
enum Constraint {
    Fixed(f32),
    Ratio(usize),
}

pub fn main() {
    let ingredients = HashMap::from([
        ("rice", 3.6),
        ("ghee", 9.),
        ("chicken", 1.1),
        ("mint", 1.),
        ("tomato", 0.18),
        ("onion", 0.4),
    ]);
    let imap = HashMap::from([
        ("rice", Constraint::Ratio(1)),
        ("chicken", Constraint::Fixed(250.)),
        ("ghee", Constraint::Fixed(2.5)),
        ("tomato", Constraint::Fixed(70.)),
        ("onion", Constraint::Fixed(70.)),
        ("mint", Constraint::Fixed(50.)),
    ]);
    let target_calories = 1100.;
    let mut total_calories = 0.;
    let mut total_ratios = 0;
    let mut to_solve = Vec::new();
    for i in imap.keys() {
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
        let amount = ((ratio as f32 / total_ratios as f32) * remaining_cals as f32)/cpg;
        println!("{i}: {amount:?}");
    }
}
