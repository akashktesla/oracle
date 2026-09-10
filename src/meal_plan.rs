#![allow(warnings)]
use std::collections::HashMap;
use std::collections::HashSet;

enum Constraint{
    Fixed(f32),
    Ratio(usize),
}

pub fn main(){
    let ingredients = HashMap::from([
        ("rice".to_string(),3.6),
        ("ghee".to_string(),9.),
        ("chicken".to_string(),1.1),
        ("mint".to_string(),1.),
        ("tomato".to_string(),0.18),
        ("onion".to_string(),0.4)
    ]);
    let imap = HashMap::from([
        ("rice",Constraint::Ratio(1)),
        ("chicken",Constraint::Fixed(250.)),
        ("ghee",Constraint::Fixed(2.5)),
        ("tomato",Constraint::Fixed(70.)),
        ("onion",Constraint::Fixed(70.)),
        ("mint",Constraint::Fixed(50.)),
    ]);
    let target_calories = 1000;
    for i in imap.keys(){
        println!("i: {i:?}");

    }
    
}




    
