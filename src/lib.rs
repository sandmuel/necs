#![feature(downcast_unchecked)]

mod register_traits;

#[doc(hidden)]
pub use necs_internal::*;
pub use necs_internal::World;
pub use necs_internal::{NodeTrait, NodeId, NodeRef, Node};
pub use necs_macros::node;
