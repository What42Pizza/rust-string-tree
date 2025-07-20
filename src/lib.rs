#![feature(coroutines, coroutine_trait)]
#![feature(portable_simd)]



/// The main type
pub mod string_tree;
pub use string_tree::*;
/// A reference to a node within a StringTree, allows for viewing, traversal, etc
pub mod string_tree_node;
pub use string_tree_node::*;
/// A mutable reference to a node within a StringTree, allows for viewing, traversal, editing, etc
pub mod string_tree_node_mut;
pub use string_tree_node_mut::*;
#[cfg(test)]
mod tests;

use std::{ops::{Coroutine, CoroutineState}, pin::Pin};



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
