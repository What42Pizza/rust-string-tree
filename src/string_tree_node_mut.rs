use crate::*;
use std::mem;



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
		let key_bytes = key.as_bytes();
		let mut curr_node = 0;
		for i in 0..key_bytes.len() {
			let key_byte = key_bytes[i] as usize;
			let next_node = self.ref_tree.node_pointers[curr_node as usize][key_byte] as usize;
			if next_node > 0 {
				curr_node = next_node;
				continue;
			} else {
				self.ref_tree.node_fill_counts[curr_node] += 1;
				for i in i .. key_bytes.len() - 1 {
					let key_byte = key_bytes[i] as usize;
					let next_node = self.ref_tree.node_pointers.len() as u32;
					self.ref_tree.node_pointers[curr_node][key_byte] = next_node;
					self.ref_tree.node_fill_counts.push(1);
					self.ref_tree.node_pointers.push([0; 256]);
					self.ref_tree.node_parents.push((curr_node as u32, key_byte as u8));
					self.ref_tree.node_values.push(None);
					curr_node = next_node as usize;
				}
				let key_byte = key_bytes[key_bytes.len() - 1] as usize;
				let next_node = self.ref_tree.node_pointers.len() as u32;
				self.ref_tree.node_pointers[curr_node][key_byte] = next_node;
				self.ref_tree.node_pointers.push([0; 256]);
				self.ref_tree.node_fill_counts.push(0);
				self.ref_tree.node_parents.push((curr_node as u32, key_byte as u8));
				self.ref_tree.node_values.push(Some(value));
				return None;
			}
		}
		let mut output = Some(value);
		mem::swap(&mut self.ref_tree.node_values[curr_node as usize], &mut output);
		output
	}
	
	/// Steps further into the tree, removes the value at that node, and returns the previous value if it exists
	/// 
	/// This also removes any unneeded nodes to ensure lowest ram usage
	pub fn remove(&mut self, key: impl AsRef<str>) -> Option<T> {
		let key = key.as_ref();
		let key_bytes = key.as_bytes();
		let Some(index) = self.get_index_of_key(key_bytes) else {return None;};
		let mut output = None;
		mem::swap(&mut self.ref_tree.node_values[index as usize], &mut output);
		if self.ref_tree.node_fill_counts[index as usize] > 0 {return output;}
		let mut end_node = index as usize;
		let mut byte_to_next = 0;
		loop {
			// only update the node's data if the node should be kept
			if self.ref_tree.node_fill_counts[end_node] > 1 || self.ref_tree.node_values[end_node].is_some() || end_node == 0 {
				// note: this cannot be run in the first iteration because of the >0 check above
				self.ref_tree.node_fill_counts[end_node] -= 1;
				self.ref_tree.node_pointers[end_node][byte_to_next as usize] = 0;
				break;
			}
			if end_node == 0 {break;}
			self.ref_tree.node_pointers.swap_remove(end_node);
			self.ref_tree.node_fill_counts.swap_remove(end_node);
			let parent_data = self.ref_tree.node_parents.swap_remove(end_node);
			self.ref_tree.node_values.swap_remove(end_node);
			if end_node != self.ref_tree.node_pointers.len() { // if a swap did occur, the tree needs to be updated
				let (swapped_parent, index_within_swapped_parent) = self.ref_tree.node_parents[end_node];
				self.ref_tree.node_pointers[swapped_parent as usize][index_within_swapped_parent as usize] = end_node as u32;
				for child_index in self.ref_tree.node_pointers[end_node] {
					if child_index == 0 {continue;}
					//assert_ne!(child_index, end_node as u32);
					self.ref_tree.node_parents[child_index as usize].0 = end_node as u32;
				}
			}
			//assert!((parent_data.0 as usize) < self.ref_tree.node_pointers.len());
			(end_node, byte_to_next) = (parent_data.0 as usize, parent_data.1);
		}
		output
	}
	
	/// Steps further into the tree and returns a new mutable node reference (or None)
	pub fn step(&mut self, key: impl AsRef<str>) -> Option<StringTreeNodeMut<'a, T>> {
		let key = key.as_ref().as_bytes();
		let index = self.get_index_of_key(key)?;
		// SAFETY: this is to get around lifetime issues, and this exact same logic is considered safe in `StringTreeNode::step()`
		Some(Self {
			ref_tree: unsafe { &mut *(self.ref_tree as *mut StringTree<T>) },
			index,
		})
	}
	/// Steps further into the tree and returns a new mutable node reference (or an error)
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
