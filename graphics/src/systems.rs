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

pub use bounds::Bounds;
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
pub use slotmap::KeyData;
pub use static_vbo::*;
pub use system::*;
pub use vbo::*;

pub(crate) use ahash::{AHashMap, AHashSet, AHasher};

pub(crate) type ABuildHasher = std::hash::BuildHasherDefault<AHasher>;
pub(crate) type AIndexSet<K> = indexmap::IndexSet<K, ABuildHasher>;

#[derive(Copy, Clone, Debug)]
pub enum CameraType {
    None,
    ControlView,
    ControlViewWithScale,
    ManualView,
    ManualViewWithScale,
}

#[derive(Copy, Clone, Debug)]
pub enum FlipStyle {
    None,
    Horizontal,
    Vertical,
    Both,
}
