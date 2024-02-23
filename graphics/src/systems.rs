mod bounds;
mod buffer;
mod device;
mod draw_order;
mod instance_buffer;
mod layout;
mod pass;
mod pipelines;
mod renderer;
mod static_vbo;
mod system;
mod vbo;

pub use bounds::{Bounds, WorldBounds};
pub use buffer::{
    AsBufferPass, Buffer, BufferData, BufferLayout, BufferPass, BufferStore,
};
pub use device::*;
pub use draw_order::{DrawOrder, DrawType, Index, OrderedIndex};
pub use instance_buffer::*;
pub use layout::*;
pub use pass::*;
pub use pipelines::*;
pub use renderer::*;
pub use static_vbo::*;
pub use system::*;
pub use vbo::*;

pub(crate) type FxBuildHasher =
    std::hash::BuildHasherDefault<ritehash::FxHasher>;
pub(crate) type FxHashMap<K, V> =
    std::collections::HashMap<K, V, FxBuildHasher>;
