use crate::*;



/// Represents a mutable node within a StringTree
pub struct StringTreeNode<'a, T> {
	pub(crate) ref_tree: &'a StringTree<T>,
	pub(crate) index: u32,
}

impl<'a, T> StringTreeNode<'a, T> {
	
	/// Steps further through the tree and returns the value at the desired position (or None)
	pub fn value_at(&'a self, key: impl AsRef<str>) -> Option<&'a T> {
		let key = key.as_ref().as_bytes();
		let index = self.get_index_of_key(key)?;
		self.ref_tree.node_values[index as usize].as_ref()
	}
	/// Steps further through the tree and returns the value at the desired position (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_value_at(&'a self, key: impl AsRef<str>) -> Result<&'a T, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path().to_string() + key)?;
		self.ref_tree.node_values[index as usize].as_ref().ok_or_else(|| self.path().to_string() + key)
	}
	
	/// Steps further through the tree and returns a new node (or None)
	pub fn step(&'a self, key: impl AsRef<str>) -> Option<StringTreeNode<'a, T>> {
		let key = key.as_ref().as_bytes();
		let index = self.get_index_of_key(key)?;
		Some(Self {
			ref_tree: self.ref_tree,
			index,
		})
	}
	/// Steps further through the tree and returns a new node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_step(&'a self, key: impl AsRef<str>) -> Result<StringTreeNode<'a, T>, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path().to_string() + key)?;
		Ok(Self {
			ref_tree: self.ref_tree,
			index,
		})
	}
	
	fn get_index_of_key(&self, key: &[u8]) -> Option<u32> {
		let mut curr_index = self.index;
		for curr_byte in key {
			let curr_pointers = self.ref_tree.node_pointers[curr_index as usize];
			curr_index = curr_pointers[*curr_byte as usize];
			if curr_index == 0 {return None;}
		}
		Some(curr_index)
	}
	
	/// Returns the value at this node (or None)
	pub fn value(&self) -> Option<&T> {
		self.ref_tree.node_values[self.index as usize].as_ref()
	}
	/// Returns the value at this node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn value_result(&self) -> Result<&'_ T, String> {
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
	pub fn children(&'a self) -> impl Iterator<Item = StringTreeNode<'a, T>> {
		IterableCoroutine(#[coroutine] || {
			let node_0 = self.index;
			// single-byte chars
			for i in 0b00000000..=0b01111111 {
				let node_1 = self.ref_tree.node_pointers[node_0 as usize][i];
				if node_1 == 0 {continue;}
				// SAFETY: this result is guaranteed to be a different section of the tree from all other yields, similar to Vec's `split_at_mut`
				yield StringTreeNode {
					ref_tree: self.ref_tree,
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
					yield StringTreeNode {
						ref_tree: self.ref_tree,
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
						yield StringTreeNode {
							ref_tree: self.ref_tree,
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
							yield StringTreeNode {
								ref_tree: self.ref_tree,
								index: node_4,
							};
						}
					}
				}
			}
		})
	}
	
	/// Effectively turns this (view of a) node into a mutable (view of a) node
	pub fn to_mut(&self, tree: &'a mut StringTree<T>) -> StringTreeNodeMut<'a, T> {
		StringTreeNodeMut {
			ref_tree: tree,
			index: self.index,
		}
	}
	
}
