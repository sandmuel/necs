#![feature(downcast_unchecked)]

mod register_traits;

pub use necs_internal::World;
#[doc(hidden)]
pub use necs_internal::*;
pub use necs_internal::{Node, NodeId, NodeRef, NodeTrait};
pub use necs_macros::node;
