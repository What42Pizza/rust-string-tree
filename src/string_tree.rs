use crate::*;
//use std::{fs::read, io::Write};
use smallvec::SmallVec;



/// A [trie](https://en.wikipedia.org/wiki/Trie) that maps strings to values. You can insert and remove items, view and edit any node, and traverse up and down the tree.
/// 
/// To traverse this tree node-by-node, you must call `StringTree::root_node()` or `StringTree::root_node_mut()`.
pub struct StringTree<T> {
	pub(crate) node_pointers: Vec<SmallVec<[(u8, u32); 4]>>, // this could technically use a smaller u32 array but the way that it is right now probably eliminates bounds checking
	pub(crate) node_fill_counts: Vec<u8>, // this technically can overflow, but even if it does, nothing bad happens (because a node is only removed when this value decreases to 0, and it can never overflow to anything above 0 because 256 is the maximum)
	pub(crate) node_stubs: Vec<[u8; 16]>,
	pub(crate) node_parents: Vec<(u32, u8)>, // (parent index, index within parent)
	pub(crate) node_values: Vec<Option<T>>,
}

impl<T> StringTree<T> {
	/// Creates a new, empty StringTree
	pub fn new() -> Self {
		Self {
			node_pointers: vec!(SmallVec::new()),
			node_fill_counts: vec!(0),
			node_stubs: vec!([0; 16]),
			node_parents: vec!((0, 0)),
			node_values: vec!(None),
		}
	}
	/// Creates a new StringTree with a given list of key/value pairs
	pub fn from<S: AsRef<str>, I: IntoIterator<Item = (S, T)>>(source: I) -> Self {
		let mut output = Self::new();
		for (key, value) in source {
			output.insert(key, value);
		}
		output
	}
	/// Inserts a key/value pair into the tree, and returns the previous value if it exists
	pub fn insert(&mut self, key: impl AsRef<str>, value: T) -> Option<T> {
		self.root_node_mut().set(key, value)
	}
	/// Gets a value from a given key (or None)
	pub fn get(&self, key: impl AsRef<str>) -> Option<&T> {
		self.root_node().get(key)
	}
	/// Gets a value from a given key (or None)
	pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut T> {
		self.root_node_mut().get_mut(key)
	}
	/// Removes and returns a value from a given key (or None if there was no value at the given key)
	pub fn remove(&mut self, key: impl AsRef<str>) -> Option<T> {
		self.root_node_mut().remove(key)
	}
	/// Steps further into the tree and returns a node reference (or None)
	pub fn step<'a>(&'a self, key: impl AsRef<str>) -> Option<StringTreeNode<'a, T>> {
		self.root_node().step(key)
	}
	/// Steps further into the tree and returns a mutable node reference (or None)
	pub fn step_mut<'a>(&'a mut self, key: impl AsRef<str>) -> Option<StringTreeNodeMut<'a, T>> {
		self.root_node_mut().step(key)
	}
	pub const fn root_node_mut<'a>(&'a mut self) -> StringTreeNodeMut<'a, T> {
		StringTreeNodeMut {
			ref_tree: self,
			index: 0,
		}
	}
	pub const fn root_node<'a>(&'a self) -> StringTreeNode<'a, T> {
		StringTreeNode {
			ref_tree: self,
			index: 0,
		}
	}
}

impl<T: Clone> Clone for StringTree<T> {
	fn clone(&self) -> Self {
		Self {
			node_pointers: self.node_pointers.clone(),
			node_fill_counts: self.node_fill_counts.clone(),
			node_stubs: self.node_stubs.clone(),
			node_parents: self.node_parents.clone(),
			node_values: self.node_values.clone(),
		}
	}
}

//impl<T: std::fmt::Debug> StringTree<T> {
//	#[allow(unused)]
//	pub(crate) fn print_self(&self) {
//		for i in 0..self.node_pointers.len() {
//			print!("{i}: ({}, {}) {:?} {} [", self.node_parents[i].0, self.node_parents[i].1 as char, self.node_values[i], self.node_fill_counts[i]);
//			for (i, v) in self.node_pointers[i].iter().enumerate() {
//				if *v == 0 {continue;}
//				print!(" {}: {v}", i as u8 as char);
//			}
//			println!("]");
//		}
//	}
//}

//#[allow(unused)]
//impl StringTree<u64> {
//	pub(crate) fn save_self(&self) {
//		let mut output = std::fs::File::create("saved_tree.dat").unwrap();
//		for i in 0..self.node_pointers.len() {
//			// pointers
//			for pointer in &self.node_pointers[i] {
//				output.write_all(&pointer.to_le_bytes()).unwrap();
//			}
//			// parent index
//			output.write_all(&self.node_parents[i].0.to_le_bytes()).unwrap();
//			// value
//			output.write_all(&self.node_values[i].unwrap_or(0).to_le_bytes()).unwrap();
//			// index within parent
//			output.write_all(&[self.node_parents[i].1]).unwrap();
//			// pointers fill count
//			output.write_all(&[self.node_fill_counts[i]]).unwrap();
//			// value is_some (plus alignment byte)
//			output.write_all(if self.node_values[i].is_some() {&[1, 0]} else {&[0, 0]}).unwrap();
//		}
//	}
//	pub(crate) fn load_self() -> Self {
//		let input = read("saved_tree.dat").unwrap();
//		let mut node_pointers = vec!();
//		let mut node_fill_counts = vec!();
//		let mut node_parents = vec!();
//		let mut node_values = vec!();
//		unsafe {
//			for mut chunk in input.chunks(256 * 4 + 4 + 8 + 1 + 1 + 1 + 1) {
//				let mut pointers = [0; 256];
//				for i in 0..256 {
//					pointers[i] = *(chunk.as_ptr() as *const u32);
//					chunk = &chunk[4..];
//				}
//				let parent_index = *(chunk.as_ptr() as *const u32);
//				chunk = &chunk[4..];
//				let value = *(chunk.as_ptr() as *const u64);
//				chunk = &chunk[8..];
//				let index_within_parent = chunk[0];
//				chunk = &chunk[1..];
//				let pointers_fill_count = chunk[0];
//				chunk = &chunk[1..];
//				let value_is_some = chunk[0];
//				chunk = &chunk[1..];
//				chunk = &chunk[1..];
//				node_pointers.push(pointers);
//				node_fill_counts.push(pointers_fill_count);
//				node_parents.push((parent_index, index_within_parent));
//				node_values.push(if value_is_some == 1 {Some(value)} else {None});
//			}
//		}
//		Self {
//			node_pointers,
//			node_fill_counts,
//			node_parents,
//			node_values
//		}
//	}
//}
