#![allow(warnings)]

use std::collections::HashSet;
struct Meal{
    ingredients: Vec<Ingredient>
}
impl Meal{
    fn new(ingredients:Vec<Ingredient>)->Self{
        return Meal{
            ingredients
        }
    }
}

struct Ingredient{
    name:String,
    calories:f32, //per 1g
}
impl Ingredient{
    fn new(name:String,calories:f32)->Self{
        return Ingredient{
            name,
            calories,
        }
    }
}

enum Constraint{
    Fixed(f32),
    Ratio(f32)
}

Struct MealSovler{
    ingredients: HashMap<Ingredient,Constraint>
}

pub fn main(){
    let rice = Ingredient::new("rice".to_string(),3.6);
    let ghee = Ingredient::new("ghee".to_string(),9.);
    let chicken  = Ingredient::new("chicken".to_string(),1.1);
    let mint = Ingredient::new("mint".to_string(),1.);
    let tomato = Ingredient::new("tomato".to_string(),0.18);
    let onion = Ingredient::new("onion".to_string(),0.40);
    let ingredients = vec![rice,ghee,chicken,mint,tomato,onion];

}
