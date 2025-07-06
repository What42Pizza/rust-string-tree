use std::collections::HashMap;

use rand::{distr::Alphanumeric, Rng};



#[test]
fn fuzz_test() {
	
	let mut hashmap = HashMap::new();
	let mut string_tree = crate::StringTree::new();
	let mut all_keys = vec!();
	
	let mut rnd = rand::rng();
	for _ in 0..1000 {
		let key = (&mut rnd)
			.sample_iter(&Alphanumeric)
			.take(16)
			.map(char::from)
			.collect::<String>();
		let value = rnd.random::<u64>();
		all_keys.push(key.to_string());
		hashmap.insert(key.to_string(), value);
		string_tree.insert(key, value);
		if all_keys.len() > 1000 {
			let key = all_keys.remove(rnd.random_range(..1000u64) as usize);
			let value_hashmap = hashmap.remove(&key);
		}
	}
	
}
