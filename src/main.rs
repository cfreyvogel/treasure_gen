use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{de, Deserialize};
fn main() {
    let feels = read_items::<GenericItem>("wine_feels.csv").unwrap();
    let notes = read_items::<GenericItem>("wine_notes.csv").unwrap();
    let containers = read_items::<LiquidContainer>("liquid_containers.csv").unwrap();
    let wine_gen = WineGenerator {
        notes_vec: notes,
        feels_vec: feels,
        container_vec: containers,
    };
    let gem_gen = GemGenerator::new();
    // let some_wine: Wine = wine_gen.create_wine();
    for _ in 1..20 {
        let gem = &gem_gen.create_random_gem();
        println!("{}", gem.as_string());
    }
}

fn read_items<T: de::DeserializeOwned>(filename: &str) -> csv::Result<Vec<T>> {
    let mut items: Vec<T> = Vec::new();
    let mut rdr = csv::Reader::from_path(filename)?;
    for result in rdr.deserialize() {
        let item: T = result?;
        items.push(item);
    }
    Ok(items)
}

fn choose_generic_items(
    ls: &Vec<GenericItem>,
    expools: &mut Vec<ExPool>,
    num: usize,
) -> Vec<GenericItem> {
    let mut items: Vec<GenericItem> = Vec::new();
    let dist = WeightedIndex::new(ls.iter().map(|x| x.weight)).unwrap();
    while items.len() < num {
        let choice: GenericItem = ls[dist.sample(&mut thread_rng())].clone();
        let expool_index: Option<usize> = choice.get_expool_index(expools);
        let excluded: bool = items.contains(&choice)
            || (expool_index.is_some() && expools[expool_index.unwrap()].is_full());
        if !excluded {
            items.push(choice);
            if expool_index.is_some() {
                expools[expool_index.unwrap()].increment();
            }
        }
    }
    items
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct GenericItem {
    name: String,
    value_mod: f64,
    weight: f64,
    ex_pool: Option<String>,
}

impl GenericItem {
    fn get_expool_index(&self, expools: &mut Vec<ExPool>) -> Option<usize> {
        if self.ex_pool.is_none() {
            // short circuit
            return None;
        }
        let mut index: usize = 0;
        let mut found: bool = false;
        for i in 0..expools.len() {
            if &expools[i].name == self.ex_pool.as_ref().unwrap() {
                index = i;
                found = true;
            }
        }
        match found {
            false => None,
            true => Some(index),
        }
    }
}

#[derive(Debug, Clone)]
struct ExPool {
    name: String,
    max_count: i32,
    count: i32,
}
impl ExPool {
    fn is_full(&self) -> bool {
        self.count == self.max_count
    }
    fn increment(&mut self) {
        if !self.is_full() {
            self.count += 1;
        }
    }
}
impl PartialEq for ExPool {
    fn eq(&self, rhs: &ExPool) -> bool {
        self.name == rhs.name
    }
}

fn create_mexpool(name: String) -> ExPool {
    ExPool {
        name,
        max_count: 1,
        count: 0,
    }
}
fn expool_vec_contains_item(ls: &Vec<ExPool>, name: String) -> bool {
    let mut found = false;
    for pool in ls {
        found = found || pool.name == name
    }
    found
}

fn auto_populate_mexpools(items: &Vec<GenericItem>) -> Vec<ExPool> {
    let mut mexpool: Vec<ExPool> = Vec::new();
    for item in items.iter() {
        if item.ex_pool.is_some()
            && !expool_vec_contains_item(&mexpool, item.ex_pool.clone().unwrap())
        {
            println!("created pool: {}", item.ex_pool.clone().unwrap());
            let pool: ExPool = create_mexpool(item.ex_pool.clone().unwrap());
            mexpool.push(pool);
        }
    }
    mexpool
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct LiquidContainer {
    name: String,
    value_mod: f64,
    weight: f64,
    oz: isize,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct Wine {
    notes: Vec<GenericItem>,
    feels: Vec<GenericItem>,
    container: LiquidContainer,
    base_value: isize,
    color: String,
}

impl Wine {
    fn total_value(&self) -> f64 {
        self.notes.iter().map(|x| x.value_mod).product::<f64>()
            * self.feels.iter().map(|x| x.value_mod).product::<f64>()
            * self.container.value_mod
            * self.container.oz as f64
            * self.base_value as f64
    }
}

struct WineGenerator {
    notes_vec: Vec<GenericItem>,
    feels_vec: Vec<GenericItem>,
    container_vec: Vec<LiquidContainer>,
}

impl WineGenerator {
    fn create_wine(&self) -> Wine {
        let mut rng = thread_rng();
        let dist = WeightedIndex::new(self.container_vec.iter().map(|x| x.weight)).unwrap();
        let container = self.container_vec[dist.sample(&mut thread_rng())].clone();
        let mut notes_mexpool: Vec<ExPool> = auto_populate_mexpools(&self.notes_vec);
        let mut feels_mexpool: Vec<ExPool> = auto_populate_mexpools(&self.feels_vec);
        let notes: Vec<GenericItem> =
            choose_generic_items(&self.notes_vec, &mut notes_mexpool, rng.gen_range(1..4));
        let feels: Vec<GenericItem> =
            choose_generic_items(&self.feels_vec, &mut feels_mexpool, rng.gen_range(1..4));
        let color: String = ["white".to_string(), "red".to_string()]
            .choose(&mut rng)
            .unwrap()
            .clone();
        Wine {
            notes,
            feels,
            container,
            base_value: 1,
            color,
        }
    }
}
#[derive(Debug, Clone)]
struct Gem {
    mineral_type: GemType,
    base_value: f64,
    cut: GemAttribute,
    size: GemAttribute,
    quality: GemAttribute,
}
impl Gem {
    fn get_value(self) -> f64 {
        let mut value: isize = self.mineral_type.value_category;
        value += self.cut.value_category_delta;
        value += self.size.value_category_delta;
        value += self.quality.value_category_delta;
        if value < 6 {
            return rand::thread_rng().gen_range(0.1..5.0);
        }
        if value > 17 {
            value = 17;
        }
        let mut value_categories: Vec<_> = Vec::new();
        value_categories.push(1..1);
        value_categories.push(1..1);
        value_categories.push(1..1);
        value_categories.push(1..1);
        value_categories.push(1..1);
        value_categories.push(1..25);
        value_categories.push(25..75);
        value_categories.push(75..250);
        value_categories.push(250..750);
        value_categories.push(750..2500);
        value_categories.push(2500..10000);
        value_categories.push(10000..20000);
        value_categories.push(20000..40000);
        value_categories.push(40000..80000);
        value_categories.push(80000..200000);
        value_categories.push(200000..400000);
        value_categories.push(400000..800000);
        value_categories.push(800000..1000000);
        return rand::thread_rng().gen_range(value_categories[value as usize].clone()) as f64;
    }
    fn as_string(&self) -> String {
        format!(
            "This is a {}cut {}. It is {} sized and of {} clarity. It is worth {} GP.",
            self.cut.name,
            self.mineral_type.name.to_lowercase(),
            self.size.name.to_lowercase(),
            self.quality.name.to_lowercase(),
            self.clone().get_value()
        )
    }
}
#[derive(Debug, Deserialize, PartialEq, Clone)]
struct GemAttribute {
    name: String,
    value_category_delta: isize,
    weight: f64,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct GemType {
    name: String,
    value_min: isize,
    value_max: isize,
    value_category: isize,
    weight: f64,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct GemGenerator {
    gem_types: Vec<GemType>,
    gem_cuts: Vec<GemAttribute>,
    gem_clarities: Vec<GemAttribute>,
    gem_sizes: Vec<GemAttribute>,
}
impl GemGenerator {
    fn new() -> GemGenerator {
        let gem_types = read_items::<GemType>("gem_types.csv").unwrap();
        let gem_cuts = read_items::<GemAttribute>("gem_cuts.csv").unwrap();
        let gem_clarities = read_items::<GemAttribute>("gem_qualities.csv").unwrap();
        let gem_sizes = read_items::<GemAttribute>("gem_sizes.csv").unwrap();
        GemGenerator {
            gem_types,
            gem_cuts,
            gem_clarities,
            gem_sizes,
        }
    }
    fn create_random_gem(&self) -> Gem {
        let gem_type = choose_random_gem_type(&self.gem_types);
        let value: f64 = thread_rng().gen_range(gem_type.value_min..gem_type.value_max) as f64;
        Gem {
            mineral_type: gem_type,
            base_value: value,
            cut: choose_random_gem_attribute(&self.gem_cuts),
            size: choose_random_gem_attribute(&self.gem_sizes),
            quality: choose_random_gem_attribute(&self.gem_clarities),
        }
    }
}
fn choose_random_gem_type(ls: &Vec<GemType>) -> GemType {
    let dist = WeightedIndex::new(ls.iter().map(|x| x.weight)).unwrap();
    ls[dist.sample(&mut thread_rng())].clone()
}
fn choose_random_gem_attribute(ls: &Vec<GemAttribute>) -> GemAttribute {
    let dist = WeightedIndex::new(ls.iter().map(|x| x.weight)).unwrap();
    ls[dist.sample(&mut thread_rng())].clone()
}
