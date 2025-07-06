#![feature(coroutines, coroutine_trait)]



/// A view of a node, allows for viewing, traversal, etc
pub mod string_tree_node;
pub use string_tree_node::*;
/// A mutable view of a node, allows for viewing, traversal, editing, etc
pub mod string_tree_node_mut;
pub use string_tree_node_mut::*;
#[cfg(test)]
mod tests;

use std::{ops::{Coroutine, CoroutineState}, pin::Pin};



/// A tree structure specifically designed for deserializing an ecf::File into structs marked with `#[derive(ToFromEcf)]`
/// 
/// This struct only holds the data, to view and traverse this tree you must call `StringTree::root_node()`
pub struct StringTree<T> {
	// note: this could technically use a smaller u32 array but the way that it is right now probably eliminates bounds checking
	node_pointers: Vec<[u32; 256]>,
	node_values: Vec<Option<T>>,
	all_paths: Vec<String>,
	node_paths: Vec<(u32, u32)>, // (path_id, depth / slice len)
}

impl<T> StringTree<T> {
	/// Creates a new, empty StringTree
	pub fn new() -> Self {
		Self {
			node_pointers: vec!([0; 256]),
			node_values: vec!(None),
			all_paths: vec!(),
			node_paths: vec!((0, 0)),
		}
	}
	/// Creates a new StringTree with a given list of key/value pairs
	pub fn from<S: ToString, I: IntoIterator<Item = (S, T)>>(source: I) -> Self {
		let mut output = Self {
			node_pointers: vec!([0; 256]),
			node_values: vec!(None),
			all_paths: vec!(),
			node_paths: vec!((0, 0)),
		};
		for (key, value) in source {
			output.insert(key, value);
		}
		output
	}
	/// Inserts a key/value pair into the tree
	pub fn insert(&mut self, key: impl ToString, value: T) {
		let key = key.to_string();
		let key_bytes = key.as_bytes();
		let mut index = 0;
		let mut iter = key_bytes.iter().enumerate();
		loop {
			let Some((i, byte)) = iter.next() else {break;};
			let byte = *byte as usize;
			let next_index = self.node_pointers[index as usize][byte];
			if next_index == 0 {
				// if it starts to create a new node then all subsequent nodes will also be new
				let path_id = self.all_paths.len() as u32;
				let (mut i, mut byte) = (i as u32, byte);
				'inner: loop {
					let next_index = self.node_pointers.len() as u32;
					self.node_pointers[index as usize][byte] = next_index;
					self.node_pointers.push([0; 256]);
					self.node_values.push(None);
					self.node_paths.push((path_id, i));
					let Some((next_i, next_byte)) = iter.next() else {break 'inner;};
					(i, byte) = (next_i as u32, *next_byte as usize);
				}
				self.all_paths.push(key);
				unsafe {
					// SAFETY: there will always be values in `self.node_values` because of the `self.node_values.push(None)` above
					*self.node_values.last_mut().unwrap_unchecked() = Some(value);
				}
				return;
			}
			index = next_index;
		}
		self.node_values[index as usize] = Some(value);
	}
	pub fn root_node_mut<'a>(&'a mut self) -> StringTreeNodeMut<'a, T> {
		StringTreeNodeMut {
			ref_tree: self,
			index: 0,
		}
	}
	pub fn root_node<'a>(&'a self) -> StringTreeNode<'a, T> {
		StringTreeNode {
			ref_tree: self,
			index: 0,
		}
	}
}



/// Utility wrapper that allows coroutines to be iterated
pub struct IterableCoroutine<C>(C);

impl<C, Y> Iterator for IterableCoroutine<C>
where
	C: Coroutine<Yield = Y, Return = ()> + Unpin,
	Y: Sized,
{
	type Item = Y;
	fn next(&mut self) -> Option<Self::Item> {
		match Pin::new(&mut self.0).resume(()) {
			CoroutineState::Yielded(val) => Some(val),
			CoroutineState::Complete(()) => None,
		}
	}
}



#[test]
fn test() {
	let mut tree = StringTree::from([("", 0)].into_iter());
	let mut node = tree.root_node_mut();
	let mut _node_2 = node.step("");
}
