use std::{collections::HashMap, fs, panic};
use rand::{distr::Alphanumeric, Rng};



//#[test]
//fn insert_get() {
//	let mut string_tree = crate::StringTree::new();
//	string_tree.insert("test", 10);
//	assert_eq!(string_tree.get("test"), Some(&10));
//	string_tree.insert("testing", 15);
//	assert_eq!(string_tree.get("testing"), Some(&15));
//	assert_eq!(string_tree.step("test").map(|node| node.value()), Some(Some(&10)));
//	assert_eq!(string_tree.step("test").map(|node| node.step("ing").map(|node| node.value())), Some(Some(Some(&15))));
//	assert_eq!(string_tree.step("test").map(|node| node.path()), Some(String::from("test")));
//	assert_eq!(string_tree.step("testing").map(|node| node.path()), Some(String::from("testing")));
//}



//#[test]
//fn remove() {
//	let mut string_tree = crate::StringTree::new();
	
//	println!("Testing basic removal...");
//	string_tree.insert("test", 10);
//	string_tree.insert("testing", 15);
//	assert_eq!(string_tree.remove("test"), Some(10));
//	assert_eq!(string_tree.get("test"), None);
//	assert_eq!(string_tree.remove("testing"), Some(15));
//	assert_eq!(string_tree.get("testing"), None);
	
//	println!("Testing basic removal (reversed)...");
//	string_tree.insert("test", 10);
//	string_tree.insert("testing", 15);
//	assert_eq!(string_tree.remove("testing"), Some(15));
//	assert_eq!(string_tree.get("testing"), None);
//	assert_eq!(string_tree.remove("test"), Some(10));
//	assert_eq!(string_tree.get("test"), None);
	
//}



use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyData {
    map: HashMap<String, u64>,
}

#[allow(static_mut_refs, unused)]
#[test]
fn fuzzing() {
	
	//let key = Simd::splat(target_key);
	//let keys: Simd<u8, _> = Simd::from_array(&);
	//let mask = key.lanes_eq(keys);
	
	//let hashmap = fs::read_to_string("saved_hashmap.json").unwrap();
	//let hashmap = serde_json::from_str::<MyData>(&hashmap).unwrap().map;
	//let mut string_tree_2 = crate::StringTree::load_self();
	//let mut count = 0;
	//for key in hashmap.keys() {
	//	count += 1;
	//	println!("{count}");
	//	string_tree_2.remove(key);
	//}
	
	//panic::set_hook(Box::new(|_info| {
	//	unsafe {
	//		let string_tree_2 = STRING_TREE_2.as_ref().unwrap_unchecked();
	//		let hashmap = HASHMAP.as_ref().unwrap_unchecked();
	//		println!("error encountered, checking basic integrity...");
	//		let mut value_count = 0;
	//		for (key, value) in hashmap.iter() {
	//			assert_eq!(Some(value), string_tree_2.get(key));
	//			value_count += 1;
	//		}
	//		let mut nodes = vec!(string_tree_2.root_node());
	//		while let Some(node) = nodes.pop() {
	//			for child in node.children() {nodes.push(child);}
	//			if let Some(value) = node.value() {
	//				let key = node.path();
	//				value_count -= 1;
	//				assert_eq!(Some(value), hashmap.get(&key));
	//			}
	//		}
	//		assert_eq!(value_count, 0);
	//		println!("done, saving...");
	//		string_tree_2.save_self();
	//		let data = serde_json::to_string_pretty(&MyData {map: hashmap.clone()}).unwrap();
	//		fs::write("saved_hashmap.json", data).unwrap();
	//		println!("done");
	//	}
	//}));
	
	//let mut hashmap = HashMap::new();
	//let mut string_tree = crate::StringTree::new();
	//let mut all_keys = vec!();
	
	//#[cfg(debug_assertions)]
	//const ITERATIONS: usize = 100000;
	//#[cfg(not(debug_assertions))]
	//const ITERATIONS: usize = 10000000;
	
	//let mut rnd = rand::rng();
	//for _ in 0..ITERATIONS {
	//	let key = (&mut rnd)
	//		.sample_iter(&Alphanumeric)
	//		.take(16)
	//		.map(char::from)
	//		.collect::<String>();
	//	let value = rnd.random::<u64>();
	//	all_keys.push(key.to_string());
	//	string_tree.insert(&key, value);
	//	hashmap.insert(key, value);
	//	if all_keys.len() > 1000 {
	//		let key = all_keys.swap_remove(rnd.random_range(..1000u64) as usize);
	//		let value_tree = string_tree.remove(&key);
	//		let value_hashmap = hashmap.remove(&key);
	//		assert_eq!(value_hashmap, value_tree);
	//	}
	//}
	
}
