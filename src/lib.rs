#![feature(coroutines, coroutine_trait)]

use std::{collections::HashMap, ops::{Coroutine, CoroutineState}, pin::Pin};



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
	/// Creates a new StringTree with a given list of key/value pairs
	pub fn new<S: ToString, I: Iterator<Item = (S, T)>>(source: I) -> Self {
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
		#[allow(invalid_reference_casting)]
		unsafe {
			// SAFETY: StringTreeNode will only ever use this StringTree as if it is non-mut, so forcing a mutable reference here is fine
			StringTreeNode (StringTreeNodeMut {
				ref_tree: &mut *(self as *const StringTree<T> as *mut StringTree<T>),
				index: 0,
			})
		}
	}
}



/// Represents a node within a StringTree
pub struct StringTreeNodeMut<'a, T> {
	ref_tree: &'a mut StringTree<T>,
	index: u32,
}

impl<'a, T> StringTreeNodeMut<'a, T> {
	
	/// Steps further through the tree and returns the value at the desired position (or None)
	pub fn value_at(&'a self, key: impl AsRef<str>) -> Option<&'a T> {
		let key = key.as_ref().as_bytes();
		let index = self.get_index_of_key(key)?;
		self.ref_tree.node_values[index as usize].as_ref()
	}
	/// Steps further through the tree and returns the value at the desired position as mut (or None)
	pub fn value_at_mut(&'a mut self, key: impl AsRef<str>) -> Option<&'a mut T> {
		let key = key.as_ref().as_bytes();
		let index = self.get_index_of_key(key)?;
		self.ref_tree.node_values[index as usize].as_mut()
	}
	/// Steps further through the tree and returns the value at the desired position (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_value_at(&'a self, key: impl AsRef<str>) -> Result<&'a T, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path().to_string() + key)?;
		self.ref_tree.node_values[index as usize].as_ref().ok_or_else(|| self.path().to_string() + key)
	}
	/// Steps further through the tree and returns the value at the desired position as mut (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_value_at_mut(&'a mut self, key: impl AsRef<str>) -> Result<&'a mut T, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path().to_string() + key)?;
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), index);}
			let ptr = self.ref_tree.node_values.as_mut_ptr().add(index as usize);
			if (*ptr).is_some() {
				Ok((*ptr).as_mut().unwrap_unchecked())
			} else {
				Err(self.path().to_string())
			}
		}
	}
	
	/// Similar to `get_value()`, but sets the value instead and (optionally) returns the previous value
	pub fn set_value_at(&'a mut self, key: impl AsRef<str>, value: T) -> Option<T> {
		let key = key.as_ref();
		if let Some(index) = self.get_index_of_key(key.as_bytes()) {
			let mut output = Some(value);
			std::mem::swap(&mut self.ref_tree.node_values[index as usize], &mut output);
			output
		} else {
			self.ref_tree.insert(key, value);
			None
		}
	}
	
	/// Steps further through the tree and returns a new node (or None)
	pub fn step(&'a mut self, key: impl AsRef<str>) -> Option<StringTreeNodeMut<'a, T>> {
		let key = key.as_ref().as_bytes();
		let index = self.get_index_of_key(key)?;
		Some(Self {
			ref_tree: &mut self.ref_tree,
			index,
		})
	}
	/// Steps further through the tree and returns a new node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_step(&'a mut self, key: impl AsRef<str>) -> Result<StringTreeNodeMut<'a, T>, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path().to_string() + key)?;
		Ok(Self {
			ref_tree: &mut self.ref_tree,
			index,
		})
	}
	
	fn get_index_of_key(&self, key: &[u8]) -> Option<u32> {
		let mut curr_index = self.index;
		for curr_byte in key {
			let curr_pointers = &self.ref_tree.node_pointers[curr_index as usize];
			curr_index = curr_pointers[*curr_byte as usize];
			if curr_index == 0 {return None;}
		}
		Some(curr_index)
	}
	
	/// Returns the value at this node (or None)
	pub fn value(&mut self) -> Option<&mut T> {
		self.ref_tree.node_values[self.index as usize].as_mut()
	}
	fn value_non_mut(&self) -> Option<&T> {
		self.ref_tree.node_values[self.index as usize].as_ref()
	}
	/// Returns the value at this node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn value_result(&mut self) -> Result<&'_ mut T, String> {
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if self.index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), self.index);}
			let ptr = self.ref_tree.node_values.as_mut_ptr().add(self.index as usize);
			if (*ptr).is_some() {
				Ok((*ptr).as_mut().unwrap_unchecked())
			} else {
				Err(self.path().to_string())
			}
		}
	}
	fn value_result_non_mut(&self) -> Result<&'_ T, String> {
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if self.index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), self.index);}
			let ptr = self.ref_tree.node_values.as_ptr().add(self.index as usize);
			if (*ptr).is_some() {
				Ok((*ptr).as_ref().unwrap_unchecked())
			} else {
				Err(self.path().to_string())
			}
		}
	}
	/// Returns the string that is needed to reach this node from the root node
	pub fn path(&'a self) -> &'a str {
		let (path_index, path_len) = self.ref_tree.node_paths[self.index as usize];
		// SAFETY: StringTreeNode-s will always point to a node that represents the ending byte of a character
		&self.ref_tree.all_paths[path_index as usize][.. path_len as usize]
	}
	
	/// Iterates over the children of this node.
	/// 
	/// Note: for multi-byte characters, this does traverse deeper into the tree to ensure that the resulting StringTreeNode will have a valid `path()`
	pub fn children(&'a mut self) -> impl Iterator<Item = StringTreeNodeMut<'a, T>> {
		IterableCoroutine(#[coroutine] || {
			let node_0 = self.index;
			// single-byte chars
			for i in 0b00000000..=0b01111111 {
				let node_1 = self.ref_tree.node_pointers[node_0 as usize][i];
				if node_1 == 0 {continue;}
				// SAFETY: this result is guaranteed to be a different section of the tree from all other yields, similar to Vec's `split_at_mut`
				yield StringTreeNodeMut {
					ref_tree: unsafe { self.ref_tree_mut() },
					index: node_1,
				};
			}
			// 2-byte chars
			for i0 in 0b11000000..=0b11011111 {
				let node_1 = self.ref_tree.node_pointers[node_0 as usize][i0];
				if node_1 == 0 {continue;}
				for i1 in 0b10000000..=0b10111111 {
					let node_2 = self.ref_tree.node_pointers[node_1 as usize][i1];
					if node_2 == 0 {continue;}
					// SAFETY: this result is guaranteed to be a different section of the tree from all other yields, similar to Vec's `split_at_mut`
					yield StringTreeNodeMut {
						ref_tree: unsafe { self.ref_tree_mut() },
						index: node_2,
					};
				}
			}
			// 3-byte chars
			for i0 in 0b11100000..=0b11101111 {
				let node_1 = self.ref_tree.node_pointers[node_0 as usize][i0];
				if node_1 == 0 {continue;}
				for i1 in 0b10000000..=0b10111111 {
					let node_2 = self.ref_tree.node_pointers[node_1 as usize][i1];
					if node_2 == 0 {continue;}
					for i2 in 0b10000000..=0b10111111 {
						let node_3 = self.ref_tree.node_pointers[node_2 as usize][i2];
						if node_3 == 0 {continue;}
						// SAFETY: this result is guaranteed to be a different section of the tree from all other yields, similar to Vec's `split_at_mut`
						yield StringTreeNodeMut {
							ref_tree: unsafe { self.ref_tree_mut() },
							index: node_3,
						};
					}
				}
			}
			// 4-byte chars
			for i0 in 0b11110000..=0b11110111 {
				let node_1 = self.ref_tree.node_pointers[node_0 as usize][i0];
				if node_1 == 0 {continue;}
				for i1 in 0b10000000..=0b10111111 {
					let node_2 = self.ref_tree.node_pointers[node_1 as usize][i1];
					if node_2 == 0 {continue;}
					for i2 in 0b10000000..=0b10111111 {
						let node_3 = self.ref_tree.node_pointers[node_2 as usize][i2];
						if node_3 == 0 {continue;}
						for i3 in 0b10000000..=0b10111111 {
							let node_4 = self.ref_tree.node_pointers[node_3 as usize][i3];
							if node_4 == 0 {continue;}
							// SAFETY: this result is guaranteed to be a different section of the tree from all other yields, similar to Vec's `split_at_mut`
							yield StringTreeNodeMut {
								ref_tree: unsafe { self.ref_tree_mut() },
								index: node_4,
							};
						}
					}
				}
			}
		})
	}
	
	unsafe fn ref_tree_mut(&'a self) -> &'a mut StringTree<T> {
		unsafe {
			&mut *(self.ref_tree as *const StringTree<T> as *mut StringTree<T>)
		}
	}
	
}



pub struct StringTreeNode<'a, T>(StringTreeNodeMut<'a, T>);

impl<'a, T> StringTreeNode<'a, T> {
	
	/// Steps further through the tree and returns the value at the desired position (or None)
	pub fn value_at(&'a self, key: impl AsRef<str>) -> Option<&'a T> {
		self.0.value_at(key)
	}
	/// Steps further through the tree and returns the value at the desired position (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_value_at(&'a self, key: impl AsRef<str>) -> Result<&'a T, String> {
		self.0.try_value_at(key)
	}
	
	/// Steps further through the tree and returns a new node (or None)
	pub fn step(&'a mut self, key: impl AsRef<str>) -> Option<StringTreeNode<'a, T>> {
		self.0.step(key).map(|node| StringTreeNode (node))
	}
	/// Steps further through the tree and returns a new node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_step(&'a mut self, key: impl AsRef<str>) -> Result<StringTreeNode<'a, T>, String> {
		self.0.try_step(key).map(|node| StringTreeNode (node))
	}
	
	/// Returns the value at this node (or None)
	pub fn value(&self) -> Option<&T> {
		self.0.value_non_mut().map(|v| &*v)
	}
	/// Returns the value at this node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn value_result(&self) -> Result<&'_ T, String> {
		self.0.value_result_non_mut().map(|v| &*v)
	}
	
	/// Returns the string that is needed to reach this node from the root node
	pub fn path(&'a self) -> &'a str {
		self.0.path()
	}
	
	/// Iterates over the children of this node.
	/// 
	/// Note: for multi-byte characters, this does traverse deeper into the tree to ensure that the resulting StringTreeNode will have a valid `path()`
	pub fn children(&'a mut self) -> impl Iterator<Item = StringTreeNode<'a, T>> {
		self.0.children().map(|child| StringTreeNode (child))
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
	let mut tree = StringTree::new([("", 0)].into_iter());
	let mut node = tree.root_node_mut();
	let mut _node_2 = node.step("");
}
