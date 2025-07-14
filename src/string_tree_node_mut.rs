use crate::*;



/// A mutable reference to a node within a StringTree
pub struct StringTreeNodeMut<'a, T> {
	pub(crate) ref_tree: &'a mut StringTree<T>,
	pub(crate) index: u32,
}

impl<'a, T> StringTreeNodeMut<'a, T> {
	
	/// Steps further into the tree and returns the value at the desired position (or None)
	pub fn get(&self, key: impl AsRef<str>) -> Option<&'a T> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes())?;
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), index);}
			let ptr = self.ref_tree.node_values.as_ptr().add(index as usize);
			if (*ptr).is_some() {
				Some((*ptr).as_ref().unwrap_unchecked())
			} else {
				None
			}
		}
	}
	/// Steps further into the tree and returns the value at the desired position as mut (or None)
	pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&'a mut T> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes())?;
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), index);}
			let ptr = self.ref_tree.node_values.as_mut_ptr().add(index as usize);
			if (*ptr).is_some() {
				Some((*ptr).as_mut().unwrap_unchecked())
			} else {
				None
			}
		}
	}
	/// Steps further into the tree and returns the value at the desired position (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_get(&self, key: impl AsRef<str>) -> Result<&'a T, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path() + key)?;
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), index);}
			let ptr = self.ref_tree.node_values.as_ptr().add(index as usize);
			if (*ptr).is_some() {
				Ok((*ptr).as_ref().unwrap_unchecked())
			} else {
				Err(self.path())
			}
		}
	}
	/// Steps further into the tree and returns the value at the desired position as mut (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_get_mut(&mut self, key: impl AsRef<str>) -> Result<&'a mut T, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path() + key)?;
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), index);}
			let ptr = self.ref_tree.node_values.as_mut_ptr().add(index as usize);
			if (*ptr).is_some() {
				Ok((*ptr).as_mut().unwrap_unchecked())
			} else {
				Err(self.path())
			}
		}
	}
	
	/// Steps further into the tree, sets the value at that node, and returns the previous value if it exists
	pub fn set(&mut self, key: impl AsRef<str>, value: T) -> Option<T> {
		let key = key.as_ref();
		if let Some(index) = self.get_index_of_key(key.as_bytes()) {
			let mut output = Some(value);
			std::mem::swap(&mut self.ref_tree.node_values[index as usize], &mut output);
			output
		} else {
			// self.ref_tree.insert(key, value);
			todo!();
			None
		}
	}
	
	/// Steps further into the tree, removes the value at that node, and returns the previous value if it exists
	pub fn remove(&mut self, key: impl AsRef<str>) -> Option<T> {
		let key = key.as_ref();
		if let Some(index) = self.get_index_of_key(key.as_bytes()) {
			let mut output = Some(value);
			std::mem::swap(&mut self.ref_tree.node_values[index as usize], &mut output);
			output
		} else {
			// self.ref_tree.insert(key, value);
			todo!();
			None
		}
	}
	
	/// Steps further into the tree and returns a new node (or None)
	pub fn step(&mut self, key: impl AsRef<str>) -> Option<StringTreeNodeMut<'a, T>> {
		let key = key.as_ref().as_bytes();
		let index = self.get_index_of_key(key)?;
		// SAFETY: this is to get around lifetime issues, and this exact same logic is considered safe in `StringTreeNode::step()`
		Some(Self {
			ref_tree: unsafe { &mut *(self.ref_tree as *mut StringTree<T>) },
			index,
		})
	}
	/// Steps further into the tree and returns a new node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn try_step(&mut self, key: impl AsRef<str>) -> Result<StringTreeNodeMut<'a, T>, String> {
		let key = key.as_ref();
		let index = self.get_index_of_key(key.as_bytes()).ok_or_else(|| self.path() + key)?;
		// SAFETY: this is to get around lifetime issues, and this exact same logic is considered safe in `StringTreeNode::step()`
		Ok(Self {
			ref_tree: unsafe { &mut *(self.ref_tree as *mut StringTree<T>) },
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
	pub fn value(&mut self) -> Option<&'a mut T> {
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if self.index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), self.index);}
			let ptr = self.ref_tree.node_values.as_mut_ptr().add(self.index as usize);
			if (*ptr).is_some() {
				Some((*ptr).as_mut().unwrap_unchecked())
			} else {
				None
			}
		}
	}
	/// Returns the value at this node (or None) without mutable references
	pub fn value_non_mut(&self) -> Option<&'a T> {
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if self.index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), self.index);}
			let ptr = self.ref_tree.node_values.as_ptr().add(self.index as usize);
			if (*ptr).is_some() {
				Some((*ptr).as_ref().unwrap_unchecked())
			} else {
				None
			}
		}
	}
	/// Returns the value at this node (or an error)
	/// 
	/// The error value is the path of the current node
	pub fn value_result(&mut self) -> Result<&'a mut T, String> {
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if self.index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), self.index);}
			let ptr = self.ref_tree.node_values.as_mut_ptr().add(self.index as usize);
			if (*ptr).is_some() {
				Ok((*ptr).as_mut().unwrap_unchecked())
			} else {
				Err(self.path())
			}
		}
	}
	/// Returns the value at this node (or an error) without mutable references
	/// 
	/// The error value is the path of the current node
	pub fn value_result_non_mut(&self) -> Result<&'a T, String> {
		// SAFETY: pointer reads are safe because of the bounds checks
		// SAFETY: unwrapping is safe because of the `is_some()` check
		unsafe {
			if self.index as usize >= self.ref_tree.node_values.len() {panic!("index out of bounds: the length is {} but the index is {}", self.ref_tree.node_values.len(), self.index);}
			let ptr = self.ref_tree.node_values.as_ptr().add(self.index as usize);
			if (*ptr).is_some() {
				Ok((*ptr).as_ref().unwrap_unchecked())
			} else {
				Err(self.path())
			}
		}
	}
	
	/// Creates and returns the string that is needed to reach this node from the root node
	pub fn path(&self) -> String {
		let mut string_bytes = vec!();
		let mut i = self.index as usize;
		while i != 0 {
			let (parent_index, index_within_parent) = self.ref_tree.node_parents[i];
			string_bytes.push(index_within_parent);
			i = parent_index as usize;
		}
		string_bytes.reverse();
		unsafe {
			// SAFETY: this result should be the path of this node, which itself should be a valid string
			String::from_utf8_unchecked(string_bytes)
		}
	}
	
	/// Iterates over the children of this node.
	/// 
	/// Note: for multi-byte characters, this does traverse deeper into the tree to ensure that the resulting StringTreeNode will have a valid `path()`
	pub fn children(&mut self) -> impl Iterator<Item = StringTreeNodeMut<'a, T>> {
		IterableCoroutine(#[coroutine] || {
			let node_0 = self.index;
			// single-byte chars
			for i in 0b00000000..=0b01111111 {
				let node_1 = self.ref_tree.node_pointers[node_0 as usize][i];
				if node_1 == 0 {continue;}
				// SAFETY: this is to get around lifetime issues, and this exact same logic is considered safe in `StringTreeNode::children()`
				yield StringTreeNodeMut {
					ref_tree: unsafe { &mut *(self.ref_tree as *mut StringTree<T>) },
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
					// SAFETY: this is to get around lifetime issues, and this exact same logic is considered safe in `StringTreeNode::children()`
					yield StringTreeNodeMut {
						ref_tree: unsafe { &mut *(self.ref_tree as *mut StringTree<T>) },
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
						// SAFETY: this is to get around lifetime issues, and this exact same logic is considered safe in `StringTreeNode::children()`
						yield StringTreeNodeMut {
							ref_tree: unsafe { &mut *(self.ref_tree as *mut StringTree<T>) },
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
							// SAFETY: this is to get around lifetime issues, and this exact same logic is considered safe in `StringTreeNode::children()`
							yield StringTreeNodeMut {
								ref_tree: unsafe { &mut *(self.ref_tree as *mut StringTree<T>) },
								index: node_4,
							};
						}
					}
				}
			}
		})
	}
	
	/// Turns this mutable node reference into a regular node reference
	pub const fn as_ref(&self) -> StringTreeNode<'a, T> {
		// SAFETY: this is to get around lifetime issues, and the lifetime of the output's ref_tree is the same lifetime as self's ref_tree because it is the same value
		StringTreeNode {
			ref_tree: unsafe { &*(self.ref_tree as *const StringTree<T>) },
			index: self.index,
		}
	}
	
}
