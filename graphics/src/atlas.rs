mod allocation;
mod allocator;
mod atlas_set;

pub use allocation::Allocation;
pub use allocator::Allocator;
pub use atlas_set::AtlasSet;

use indexmap::IndexSet;

pub struct Atlas {
    // handles the space allocation of the layer.
    pub allocator: Allocator,
    //Stores each slab index the allocations exist at for this layer.
    pub allocated: IndexSet<usize>,
    // use to avoid placing newly loaded images into
    //if we are migrating images out of it.
    pub migrating: bool,
}

impl Atlas {
    pub fn new(size: u32) -> Self {
        Self {
            allocator: Allocator::new(size),
            allocated: IndexSet::new(),
            migrating: false,
        }
    }

    pub fn allocate(
        &mut self,
        width: u32,
        height: u32,
    ) -> Option<guillotiere::Allocation> {
        self.allocator.allocate(width, height)
    }

    pub fn insert_index(&mut self, index: usize) {
        self.allocated.insert(index);
    }

    pub fn clear(&mut self) {
        self.allocator.clear();
        self.allocated.clear();
        self.migrating = false;
    }

    pub fn deallocate(
        &mut self,
        index: usize,
        allocation: guillotiere::Allocation,
    ) {
        self.allocated.swap_remove(&index);
        self.allocator.deallocate(allocation);
    }

    /// Returns how many alloctions have been removed since the
    /// creation of the layer. this gets reset when the layer is purged.
    pub fn deallocations(&self) -> usize {
        self.allocator.deallocations()
    }

    pub fn start_migration(&mut self) {
        self.migrating = true;
    }
}
