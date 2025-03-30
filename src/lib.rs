use bevy::{ecs::system::Resource, utils::hashbrown::HashSet};
use rand::random_range;

/// Trait for items that can be stored and retrieved from an item pool.
pub trait PoolItem = Copy + std::hash::Hash + Eq + Sync;

/// Trait for a resource that manages a single store of items.
pub trait SingleStoreItemPool<T: PoolItem> {
    /// Returns a label for this item pool.
    fn label(&self) -> String;

    /// Returns a mutable reference to the main pool of items.
    fn pool(&mut self) -> &mut Vec<T>;

    /// Returns a mutable reference to the pool of discarded items.
    /// 
    /// A temporary storage area for items that have been removed from the main pool
    /// but might be needed again later. This allows for recycling of items without
    /// needing to re-allocate or re-initialize them.
    fn get_discard_pool(&mut self) -> &mut Vec<T>;

    /// Discards a single item by moving it from the main pool to the discard pool.
    fn discard_one(&mut self, item: T) {
        self.get_discard_pool().push(item);
    }

    /// Moves all items from the discard pool back into the main pool.
    fn recycle_discarded(&mut self) {
        let mut discard_pool = std::mem::take(self.get_discard_pool());
        // Append all recycled items into the pool.
        self.pool().append(&mut discard_pool);
    }

    /// Retrieves a specified number of unique items from the pool.
    ///
    /// Items are randomly selected from the pool. If a selected item is already
    /// in the result set, it is returned to the pool and another item is selected.
    ///
    /// # Panics
    ///
    /// Panics if the pool is empty.
    fn get_set(&mut self, mut size: u8) -> HashSet<T> {
        self.recycle_discarded();

        let store = self.pool();
        debug_assert!(!store.is_empty(), "The {} pool is empty", self.label());

        let mut remaining = store.len();

        let mut colliding = Vec::new();
        let mut values = HashSet::new();
        while size != 0 {
            let draw = store.remove(random_range(0..remaining));
            match values.contains(&draw) {
                true => colliding.push(draw),
                false => {
                    values.insert(draw);
                    size -= 1;
                    remaining -= 1;
                }
            }
        }
        store.extend(colliding.into_iter());
        values
    }

    /// Retrieves a single random item from the pool.
    ///
    /// # Panics
    ///
    /// Panics if the pool is empty.
    fn get_one(&mut self) -> T {
        let store = self.pool();
        debug_assert!(!store.is_empty(), "The {} pool is empty", self.label());

        store.remove(random_range(0..store.len()))
    }
}

/// Trait for a Bevy resource that implements the `SingleStoreItemPool` trait.
pub trait ItemPoolResource<T: PoolItem> = Resource + SingleStoreItemPool<T>;
