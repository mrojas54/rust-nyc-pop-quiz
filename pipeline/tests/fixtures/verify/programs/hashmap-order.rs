// AC-8: HashMap iteration order is seeded per process, so the order the keys
// come out in differs from run to run. The verifier must reject it.
use std::collections::HashMap;

fn main() {
    let names = ["ada", "bjarne", "grace", "ken", "linus", "rob", "barbara", "niklaus"];
    let mut ages = HashMap::new();
    for (i, name) in names.iter().enumerate() {
        ages.insert(*name, i);
    }
    let keys: Vec<_> = ages.keys().collect();
    println!("{:?}", keys);
}
