pub struct Allocator {
    allocator: guillotiere::AtlasAllocator,
    allocations: usize,
    deallocations: usize,
}

impl Allocator {
    pub fn allocate(
        &mut self,
        width: u32,
        height: u32,
    ) -> Option<guillotiere::Allocation> {
        let allocation = self
            .allocator
            .allocate(guillotiere::Size::new(width as i32, height as i32))?;

        self.allocations += 1;

        Some(allocation)
    }

    pub fn clear(&mut self) {
        self.allocator.clear();
        self.allocations = 0;
        self.deallocations = 0;
    }

    pub fn deallocate(&mut self, allocation: guillotiere::Allocation) {
        self.allocator.deallocate(allocation.id);

        self.allocations = self.allocations.saturating_sub(1);
        self.deallocations = self.deallocations.saturating_add(1);
    }

    pub fn is_empty(&self) -> bool {
        self.allocations == 0
    }

    pub fn deallocations(&self) -> usize {
        self.deallocations
    }

    pub fn new(size: u32) -> Self {
        let allocator = guillotiere::AtlasAllocator::new(
            guillotiere::Size::new(size as i32, size as i32),
        );

        Self {
            allocator,
            allocations: 0,
            deallocations: 0,
        }
    }
}
