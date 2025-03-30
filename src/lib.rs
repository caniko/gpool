#![feature(trait_alias)]

#[cfg(feature = "bevy")]
pub mod bevy;

use bevy_utils::HashSet;
use rand::random_range;

/// Trait for items that can be stored and retrieved from an item pool.
pub trait PoolItem = std::hash::Hash + Eq + Sync;

/// Trait for a resource that manages a single store of items.
pub trait ItemPool<T: PoolItem> {
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
    fn get_set(&mut self, mut size: usize) -> HashSet<T> {
        self.recycle_discarded();

        let store = self.pool();
        debug_assert!(!store.is_empty(), "The {} pool is empty", self.label());

        let mut remaining = store.len();

        let mut colliding = Vec::new();
        let mut values = HashSet::new();
        while size - colliding.len() > 0 {
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

#[cfg(test)]
mod test {
    use super::*; // Import items from the parent module (where your code is)

    // Concrete implementation for testing
    #[derive(Debug)]
    struct TestPool {
        label: String,
        items: Vec<u32>, // Using u32 as a concrete PoolItem type for tests
        discarded_items: Vec<u32>,
    }

    impl TestPool {
        fn new(label: &str, initial_items: Vec<u32>) -> Self {
            TestPool {
                label: label.to_string(),
                items: initial_items,
                discarded_items: Vec::new(),
            }
        }
    }

    // Implement the trait for our test struct
    impl ItemPool<u32> for TestPool {
        fn label(&self) -> String {
            self.label.clone()
        }

        fn pool(&mut self) -> &mut Vec<u32> {
            &mut self.items
        }

        fn get_discard_pool(&mut self) -> &mut Vec<u32> {
            &mut self.discarded_items
        }
    }

    // --- Unit Tests ---

    #[test]
    fn test_label() {
        let pool = TestPool::new("MyTestPool", vec![1, 2, 3]);
        assert_eq!(pool.label(), "MyTestPool");
    }

    #[test]
    fn test_discard_one() {
        let mut pool = TestPool::new("DiscardTest", vec![10, 20, 30]);
        assert_eq!(pool.pool().len(), 3);
        assert!(pool.get_discard_pool().is_empty());

        // Note: discard_one only moves to discard, it doesn't remove from the main pool
        // in this default implementation. This might be unexpected.
        // Let's test the implemented behavior.
        pool.discard_one(20);

        // Assert item is in discard pool
        assert_eq!(pool.get_discard_pool().len(), 1);
        assert_eq!(pool.get_discard_pool()[0], 20);

        // Assert item is *still* in the main pool (based on default implementation)
        // This is important to note. If removal from main pool was intended,
        // the trait or the implementation using it would need adjustment.
        // However, get_one/get_set *do* remove from the main pool.
        assert_eq!(pool.pool().len(), 3);
        assert!(pool.pool().contains(&20));
    }

    #[test]
    fn test_recycle_discarded() {
        let mut pool = TestPool::new("RecycleTest", vec![1, 2]);
        // Manually add to discard for testing recycle logic directly
        pool.get_discard_pool().push(3);
        pool.get_discard_pool().push(4);

        assert_eq!(pool.pool().len(), 2);
        assert_eq!(pool.get_discard_pool().len(), 2);

        pool.recycle_discarded();

        assert!(pool.get_discard_pool().is_empty());
        assert_eq!(pool.pool().len(), 4);
        // Order might not be guaranteed depending on append behavior, so check contents
        let mut expected_items = vec![1, 2, 3, 4];
        let mut actual_items = pool.pool().clone();
        expected_items.sort_unstable();
        actual_items.sort_unstable();
        assert_eq!(actual_items, expected_items);
    }

    #[test]
    fn test_get_one() {
        let mut pool = TestPool::new("GetOneTest", vec![5, 15, 25]);
        let initial_len = pool.pool().len();

        let item = pool.get_one();

        // Check item was from the original pool
        assert!([5, 15, 25].contains(&item));
        // Check pool size decreased
        assert_eq!(pool.pool().len(), initial_len - 1);
        // Check the specific item is removed
        assert!(!pool.pool().contains(&item));
    }

    #[test]
    #[should_panic(expected = "The GetOneEmptyTest pool is empty")]
    fn test_get_one_empty_pool_panics() {
        let mut pool = TestPool::new("GetOneEmptyTest", vec![]);
        pool.get_one(); // This should panic
    }

    #[test]
    fn test_get_set() {
        let mut pool = TestPool::new("GetSetTest", vec![1, 2, 3, 4, 5, 6, 7]);
        let initial_len = pool.pool().len();
        let requested_size: usize = 3;

        let item_set = pool.get_set(requested_size);

        // Check set size
        assert_eq!(item_set.len(), requested_size as usize);
        // Check pool size decreased correctly
        assert_eq!(pool.pool().len(), initial_len - (requested_size as usize));

        // Check all items in the set were originally in the pool and are now removed
        let initial_items: HashSet<u32> = [1, 2, 3, 4, 5, 6, 7].iter().cloned().collect();
        for item in item_set.iter() {
            assert!(initial_items.contains(item));
            assert!(!pool.pool().contains(item));
        }
    }

    #[test]
    fn test_get_set_with_recycling() {
        let mut pool = TestPool::new("GetSetRecycleTest", vec![1, 2]);
        pool.get_discard_pool().push(3);
        pool.get_discard_pool().push(4);
        let requested_size: usize = 3;

        // Pool starts with [1, 2], discard has [3, 4]
        // get_set first recycles, pool becomes [1, 2, 3, 4] (order may vary)
        let initial_total_len = pool.pool().len() + pool.get_discard_pool().len(); // Should be 4

        let item_set = pool.get_set(requested_size);

        // Check discard pool is empty after recycling step in get_set
        assert!(pool.get_discard_pool().is_empty());
        // Check set size
        assert_eq!(item_set.len(), requested_size as usize); // Should be 3
        // Check pool size decreased correctly from the total available after recycling
        assert_eq!(pool.pool().len(), initial_total_len - (requested_size as usize)); // Should be 4 - 3 = 1

        // Check items came from the combined pool
        let combined_items: HashSet<u32> = [1, 2, 3, 4].iter().cloned().collect();
        for item in item_set.iter() {
            assert!(combined_items.contains(item));
            assert!(!pool.pool().contains(item)); // Check they were removed from the final pool state
        }
    }

    #[test]
    fn test_get_set_request_all() {
        let mut pool = TestPool::new("GetSetAllTest", vec![10, 20, 30]);
        let initial_len = pool.pool().len();
        let requested_size: usize = 3;

        let item_set = pool.get_set(requested_size);

        assert_eq!(item_set.len(), initial_len);
        assert!(pool.pool().is_empty()); // All items should be removed

        let expected_set: HashSet<u32> = [10, 20, 30].iter().cloned().collect();
        assert_eq!(item_set, expected_set);
    }

    #[test]
    #[should_panic(expected = "The GetSetEmptyTest pool is empty")]
    fn test_get_set_empty_pool_panics() {
        let mut pool = TestPool::new("GetSetEmptyTest", vec![]);
        pool.get_set(1); // This should panic
    }

    #[test]
    fn test_get_set_handles_duplicates_internally() {
        // This test is harder to make deterministic due to random draws.
        // We rely on the implementation detail that duplicates drawn are put
        // back into the pool eventually (`colliding` vector).
        // We test that if we request a size smaller than the pool, the final
        // pool state plus the result set should contain all original unique items.
        let mut pool = TestPool::new("GetSetDupTest", vec![1, 2, 3, 4, 5]);
        let initial_items: HashSet<u32> = pool.pool().iter().cloned().collect();
        let initial_len = pool.pool().len();
        let requested_size: usize = 2;

        let item_set = pool.get_set(requested_size);

        assert_eq!(item_set.len(), requested_size as usize);
        assert_eq!(pool.pool().len(), initial_len - (requested_size as usize));

        // Combine remaining pool items and the returned set
        let final_items: HashSet<u32> = pool.pool().iter().cloned().collect();
        let reconstructed_items: HashSet<u32> = item_set.union(&final_items).cloned().collect();

        // Should contain all original items
        assert_eq!(reconstructed_items, initial_items);
    }
}