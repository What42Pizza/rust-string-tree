use crate::*;



/// A [trie](https://en.wikipedia.org/wiki/Trie) that maps strings to values. You can insert and remove items, view and edit any node, and traverse up and down the tree.
/// 
/// To traverse this tree node-by-node, you must call `StringTree::root_node()` or `StringTree::root_node_mut()`.
pub struct StringTree<T> {
	pub(crate) node_pointers: Vec<[u32; 256]>, // this could technically use a smaller u32 array but the way that it is right now probably eliminates bounds checking
	pub(crate) node_fill_counts: Vec<u8>, // this technically can overflow, but even if it does, nothing bad happens (because a node is only removed when this value decreases to 0, and it can never overflow to anything above 0 because 256 is the maximum)
	pub(crate) node_values: Vec<Option<T>>,
	pub(crate) node_parents: Vec<(u32, u8)>, // (parent index, index within parent)
	pub(crate) open_nodes: Vec<u32>,
}

impl<T> StringTree<T> {
	/// Creates a new, empty StringTree
	pub fn new() -> Self {
		Self {
			node_pointers: vec!([0; 256]),
			node_values: vec!(None),
			node_parents: vec!((0, 0)),
			node_fill_counts: vec!(0),
			open_nodes: vec!(),
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
	///// Inserts a key/value pair into the tree
	//pub fn insert(&mut self, key: impl ToString, value: T) {
	//	let key = key.to_string();
	//	let key_bytes = key.as_bytes();
	//	let mut index = 0;
	//	let mut iter = key_bytes.iter().enumerate();
	//	loop {
	//		let Some((i, byte)) = iter.next() else {break;};
	//		let byte = *byte as usize;
	//		let next_index = self.node_pointers[index as usize][byte];
	//		if next_index == 0 {
	//			// if it starts to create a new node then all subsequent nodes will also be new
	//			let path_id = self.all_paths.len() as u32;
	//			let (mut i, mut byte) = (i as u32, byte);
	//			'inner: loop {
	//				let next_index = self.node_pointers.len() as u32;
	//				self.node_pointers[index as usize][byte] = next_index;
	//				self.node_pointers.push([0; 256]);
	//				self.node_values.push(None);
	//				self.node_paths.push((path_id, i));
	//				let Some((next_i, next_byte)) = iter.next() else {break 'inner;};
	//				(i, byte) = (next_i as u32, *next_byte as usize);
	//			}
	//			self.all_paths.push(key);
	//			unsafe {
	//				// SAFETY: there will always be values in `self.node_values` because of the `self.node_values.push(None)` above
	//				*self.node_values.last_mut().unwrap_unchecked() = Some(value);
	//			}
	//			return;
	//		}
	//		index = next_index;
	//	}
	//	self.node_values[index as usize] = Some(value);
	//}
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
