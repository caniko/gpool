//! # Item Pool
//!
//! A lightweight library for managing pools of reusable items with support for
//! random selection, unique set retrieval, and item recycling.
//!
//! ## Features
//!
//! - **Random Item Selection**: Efficiently retrieve random items from a pool
//! - **Unique Sets**: Get sets of unique items without duplicates
//! - **Item Recycling**: Temporarily discard items and recycle them back into the pool
//! - **Zero Dependencies**: Only uses `rand` for randomization
//!
//! ## Example
//!
//! ```rust
//! use itempool::{ItemPool, RecyclingItemPool};
//! use std::collections::HashSet;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! struct EntityId(u32);
//!
//! struct EnemyPool {
//!     available: Vec<EntityId>,
//!     discarded: Vec<EntityId>,
//! }
//!
//! impl ItemPool<EntityId> for EnemyPool {
//!     fn pool(&mut self) -> &mut Vec<EntityId> {
//!         &mut self.available
//!     }
//! }
//!
//! impl RecyclingItemPool<EntityId> for EnemyPool {
//!     fn get_discard_pool(&mut self) -> &mut Vec<EntityId> {
//!         &mut self.discarded
//!     }
//! }
//! ```

use std::collections::HashSet;

use rand::{Rng, rng};

/// Trait for items that can be stored and retrieved from an item pool.
///
/// Items must be:
/// - `Hash`: Required for storing in `HashSet` to ensure uniqueness
/// - `Eq`: Required for equality comparisons
/// - `Sync`: Required for safe concurrent access across threads
pub trait PoolItem: std::hash::Hash + Eq + Sync {}

/// Blanket implementation for all types that satisfy the constraints.
impl<T> PoolItem for T where T: std::hash::Hash + Eq + Sync {}

/// Trait for a resource that manages a single store of items.
///
/// This trait provides methods for adding items to a pool and retrieving them
/// randomly. Items are removed from the pool when retrieved.
///
/// # Type Parameters
///
/// * `T` - The type of items stored in the pool, must implement [`PoolItem`]
///
/// # Examples
///
/// ```rust
/// use itempool::ItemPool;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// struct Item(i32);
///
/// struct SimplePool {
///     items: Vec<Item>,
/// }
///
/// impl ItemPool<Item> for SimplePool {
///     fn pool(&mut self) -> &mut Vec<Item> {
///         &mut self.items
///     }
/// }
/// ```
pub trait ItemPool<T: PoolItem> {
    /// Returns a mutable reference to the main pool of items.
    ///
    /// This is the primary storage for available items that can be retrieved.
    fn pool(&mut self) -> &mut Vec<T>;

    /// Adds an item to the pool.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to add to the pool
    ///
    /// # Examples
    ///
    /// ```
    /// # use itempool::ItemPool;
    /// # #[derive(Hash, Eq, PartialEq)] struct Item;
    /// # struct Pool { items: Vec<Item> }
    /// # impl ItemPool<Item> for Pool { fn pool(&mut self) -> &mut Vec<Item> { &mut self.items } }
    /// # let mut pool = Pool { items: vec![] };
    /// pool.put(Item);
    /// ```
    fn put(&mut self, item: T) {
        self.pool().push(item);
    }

    /// Retrieves and removes one randomly selected item from the pool.
    ///
    /// # Returns
    ///
    /// * `Some(T)` - A randomly selected item from the pool
    /// * `None` - If the pool is empty
    ///
    /// # Examples
    ///
    /// ```
    /// # use itempool::ItemPool;
    /// # #[derive(Hash, Eq, PartialEq, Debug)] struct Item(i32);
    /// # struct Pool { items: Vec<Item> }
    /// # impl ItemPool<Item> for Pool { fn pool(&mut self) -> &mut Vec<Item> { &mut self.items } }
    /// # let mut pool = Pool { items: vec![Item(1), Item(2), Item(3)] };
    /// if let Some(item) = pool.remove_one() {
    ///     println!("Got item: {:?}", item);
    /// }
    /// ```
    fn remove_one(&mut self) -> Option<T> {
        let store = self.pool();
        if store.is_empty() {
            None
        } else {
            Some(store.remove(rng().random_range(0..store.len())))
        }
    }

    /// Retrieves and removes a specified number of items from the pool.
    ///
    /// Items are selected randomly. Note that uniqueness is **not** guaranteed -
    /// the same item may appear multiple times if it was in the pool multiple times.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of items to retrieve
    ///
    /// # Returns
    ///
    /// A vector containing the requested items
    ///
    /// # Panics
    ///
    /// Panics in debug mode if:
    /// * The pool is empty
    /// * `size` exceeds the number of available items
    ///
    /// # Examples
    ///
    /// ```
    /// # use itempool::ItemPool;
    /// # #[derive(Hash, Eq, PartialEq, Debug)] struct Item(i32);
    /// # struct Pool { items: Vec<Item> }
    /// # impl ItemPool<Item> for Pool { fn pool(&mut self) -> &mut Vec<Item> { &mut self.items } }
    /// # let mut pool = Pool { items: (0..10).map(Item).collect() };
    /// let items = pool.remove_many(3);
    /// assert_eq!(items.len(), 3);
    /// ```
    fn remove_many(&mut self, size: usize) -> Vec<T> {
        let store = self.pool();
        debug_assert!(!store.is_empty(), "The pool is empty");

        let mut remaining = store.len();
        debug_assert!(
            size <= remaining,
            "Requested size exceeds available items in the pool: {} <= {}",
            size,
            remaining
        );

        let mut values = Vec::with_capacity(size);
        for _ in 0..size {
            values.push(store.remove(rng().random_range(0..remaining)));
            remaining -= 1;
        }
        values
    }

    /// Retrieves and removes a specified number of **unique** items from the pool.
    ///
    /// Items are randomly selected. If a selected item is already in the result set,
    /// it is temporarily set aside and another item is selected. Items that collide
    /// are returned to the pool after selection is complete.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of unique items to retrieve
    ///
    /// # Returns
    ///
    /// A `HashSet` containing the unique items
    ///
    /// # Panics
    ///
    /// Panics in debug mode if:
    /// * The pool is empty
    /// * `size` exceeds the number of available items
    ///
    /// # Examples
    ///
    /// ```
    /// # use itempool::ItemPool;
    /// # #[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)] struct Item(i32);
    /// # struct Pool { items: Vec<Item> }
    /// # impl ItemPool<Item> for Pool { fn pool(&mut self) -> &mut Vec<Item> { &mut self.items } }
    /// # let mut pool = Pool { items: (0..10).map(Item).collect() };
    /// let items = pool.remove_set(5);
    /// assert_eq!(items.len(), 5); // All items are unique
    /// ```
    fn remove_set(&mut self, mut size: usize) -> HashSet<T> {
        let store = self.pool();
        debug_assert!(!store.is_empty(), "The pool is empty");

        let mut remaining = store.len();
        debug_assert!(
            size <= remaining,
            "Requested size exceeds available items in the pool: {} <= {}",
            size,
            remaining
        );

        let mut colliding = Vec::new();
        let mut values = HashSet::new();
        while size - colliding.len() > 0 {
            let draw = store.remove(rng().random_range(0..remaining));
            match values.contains(&draw) {
                true => colliding.push(draw),
                false => {
                    values.insert(draw);
                    size -= 1;
                    remaining -= 1;
                }
            }
        }
        store.append(&mut colliding);
        values
    }
}

/// Trait for managing item pools with discard and recycling capabilities.
///
/// This trait extends [`ItemPool`] to add support for temporarily discarding items
/// and later recycling them back into the main pool. This is useful for scenarios
/// where items need to be temporarily unavailable but should be reused later.
///
/// # Examples
///
/// ```
/// use itempool::{ItemPool, RecyclingItemPool};
/// use std::collections::HashSet;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// struct Card(u32);
///
/// struct Deck {
///     draw_pile: Vec<Card>,
///     discard_pile: Vec<Card>,
/// }
///
/// impl ItemPool<Card> for Deck {
///     fn pool(&mut self) -> &mut Vec<Card> {
///         &mut self.draw_pile
///     }
/// }
///
/// impl RecyclingItemPool<Card> for Deck {
///     fn get_discard_pool(&mut self) -> &mut Vec<Card> {
///         &mut self.discard_pile
///     }
/// }
/// ```
pub trait RecyclingItemPool<T: PoolItem>: ItemPool<T> {
    /// Returns a mutable reference to the pool of discarded items.
    ///
    /// A temporary storage area for items that have been removed from the main pool
    /// but might be needed again later. This allows for recycling of items without
    /// needing to re-allocate or re-initialize them.
    fn get_discard_pool(&mut self) -> &mut Vec<T>;

    /// Adds an item to the discard pool.
    ///
    /// Note: This method only adds the item to the discard pool. It does not
    /// remove it from the main pool. If you need to move an item from the main
    /// pool to the discard pool, you must remove it from the main pool separately.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to discard
    ///
    /// # Examples
    ///
    /// ```
    /// # use itempool::{ItemPool, RecyclingItemPool};
    /// # #[derive(Hash, Eq, PartialEq, Clone, Copy)] struct Item(i32);
    /// # struct Pool { items: Vec<Item>, discarded: Vec<Item> }
    /// # impl ItemPool<Item> for Pool { fn pool(&mut self) -> &mut Vec<Item> { &mut self.items } }
    /// # impl RecyclingItemPool<Item> for Pool { fn get_discard_pool(&mut self) -> &mut Vec<Item> { &mut self.discarded } }
    /// # let mut pool = Pool { items: vec![], discarded: vec![] };
    /// pool.discard_one(Item(1));
    /// ```
    fn discard_one(&mut self, item: T) {
        self.get_discard_pool().push(item);
    }

    /// Moves all items from the discard pool back into the main pool.
    ///
    /// After this operation, the discard pool will be empty and all previously
    /// discarded items will be available again in the main pool.
    ///
    /// # Examples
    ///
    /// ```
    /// # use itempool::{ItemPool, RecyclingItemPool};
    /// # #[derive(Hash, Eq, PartialEq, Clone, Copy)] struct Item(i32);
    /// # struct Pool { items: Vec<Item>, discarded: Vec<Item> }
    /// # impl ItemPool<Item> for Pool { fn pool(&mut self) -> &mut Vec<Item> { &mut self.items } }
    /// # impl RecyclingItemPool<Item> for Pool { fn get_discard_pool(&mut self) -> &mut Vec<Item> { &mut self.discarded } }
    /// # let mut pool = Pool { items: vec![], discarded: vec![Item(1), Item(2)] };
    /// pool.recycle_discarded();
    /// assert_eq!(pool.pool().len(), 2);
    /// assert_eq!(pool.get_discard_pool().len(), 0);
    /// ```
    fn recycle_discarded(&mut self) {
        let mut discard_pool = std::mem::take(self.get_discard_pool());
        // Append all recycled items into the pool.
        self.pool().append(&mut discard_pool);
    }

    /// Retrieves a set of unique items, automatically recycling discarded items first.
    ///
    /// This method first calls [`recycle_discarded`](Self::recycle_discarded) to ensure
    /// all discarded items are available, then retrieves the requested number of unique
    /// items using [`ItemPool::remove_set`].
    ///
    /// # Arguments
    ///
    /// * `size` - The number of unique items to retrieve
    ///
    /// # Returns
    ///
    /// A `HashSet` containing the unique items
    ///
    /// # Panics
    ///
    /// Panics in debug mode if:
    /// * The combined pool (main + discarded) is empty after recycling
    /// * `size` exceeds the total number of available items after recycling
    ///
    /// # Examples
    ///
    /// ```
    /// # use itempool::{ItemPool, RecyclingItemPool};
    /// # #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)] struct Item(i32);
    /// # struct Pool { items: Vec<Item>, discarded: Vec<Item> }
    /// # impl ItemPool<Item> for Pool { fn pool(&mut self) -> &mut Vec<Item> { &mut self.items } }
    /// # impl RecyclingItemPool<Item> for Pool { fn get_discard_pool(&mut self) -> &mut Vec<Item> { &mut self.discarded } }
    /// # let mut pool = Pool { items: (0..5).map(Item).collect(), discarded: (5..10).map(Item).collect() };
    /// let items = pool.get_set(7);
    /// assert_eq!(items.len(), 7);
    /// ```
    fn get_set(&mut self, size: usize) -> HashSet<T> {
        // Recycle discarded items before getting a set
        self.recycle_discarded();
        // Call the original get_set method
        ItemPool::remove_set(self, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct TestItem(i32);

    struct TestPool {
        items: Vec<TestItem>,
    }

    impl ItemPool<TestItem> for TestPool {
        fn pool(&mut self) -> &mut Vec<TestItem> {
            &mut self.items
        }
    }

    struct TestRecyclingPool {
        items: Vec<TestItem>,
        discarded: Vec<TestItem>,
    }

    impl ItemPool<TestItem> for TestRecyclingPool {
        fn pool(&mut self) -> &mut Vec<TestItem> {
            &mut self.items
        }
    }

    impl RecyclingItemPool<TestItem> for TestRecyclingPool {
        fn get_discard_pool(&mut self) -> &mut Vec<TestItem> {
            &mut self.discarded
        }
    }

    #[test]
    fn test_put() {
        let mut pool = TestPool { items: vec![] };
        pool.put(TestItem(1));
        pool.put(TestItem(2));
        assert_eq!(pool.pool().len(), 2);
    }

    #[test]
    fn test_remove_one_empty() {
        let mut pool = TestPool { items: vec![] };
        assert_eq!(pool.remove_one(), None);
    }

    #[test]
    fn test_remove_one_success() {
        let mut pool = TestPool {
            items: vec![TestItem(1), TestItem(2), TestItem(3)],
        };
        let item = pool.remove_one();
        assert!(item.is_some());
        assert_eq!(pool.pool().len(), 2);
    }

    #[test]
    fn test_remove_many() {
        let mut pool = TestPool {
            items: (0..10).map(TestItem).collect(),
        };
        let items = pool.remove_many(5);
        assert_eq!(items.len(), 5);
        assert_eq!(pool.pool().len(), 5);
    }

    #[test]
    fn test_remove_set_uniqueness() {
        let mut pool = TestPool {
            items: (0..10).map(TestItem).collect(),
        };
        let set = pool.remove_set(5);
        assert_eq!(set.len(), 5);
        assert_eq!(pool.pool().len(), 5);

        // Verify all items are unique
        let vec: Vec<_> = set.into_iter().collect();
        let unique_set: HashSet<_> = vec.iter().copied().collect();
        assert_eq!(vec.len(), unique_set.len());
    }

    #[test]
    fn test_remove_set_all_items() {
        let mut pool = TestPool {
            items: (0..5).map(TestItem).collect(),
        };
        let set = pool.remove_set(5);
        assert_eq!(set.len(), 5);
        assert_eq!(pool.pool().len(), 0);
    }

    #[test]
    fn test_discard_and_recycle() {
        let mut pool = TestRecyclingPool {
            items: (0..5).map(TestItem).collect(),
            discarded: vec![],
        };

        pool.discard_one(TestItem(10));
        pool.discard_one(TestItem(11));
        assert_eq!(pool.get_discard_pool().len(), 2);
        assert_eq!(pool.pool().len(), 5);

        pool.recycle_discarded();
        assert_eq!(pool.get_discard_pool().len(), 0);
        assert_eq!(pool.pool().len(), 7);
    }

    #[test]
    fn test_get_set_with_recycling() {
        let mut pool = TestRecyclingPool {
            items: (0..3).map(TestItem).collect(),
            discarded: (3..7).map(TestItem).collect(),
        };

        let set = pool.get_set(5);
        assert_eq!(set.len(), 5);
        assert_eq!(pool.get_discard_pool().len(), 0);
        assert_eq!(pool.pool().len(), 2);
    }

    #[test]
    fn test_multiple_operations() {
        let mut pool = TestRecyclingPool {
            items: (0..10).map(TestItem).collect(),
            discarded: vec![],
        };

        // Remove some items
        let _first = pool.remove_one();
        assert_eq!(pool.pool().len(), 9);

        // Remove a set
        let _set = pool.remove_set(3);
        assert_eq!(pool.pool().len(), 6);

        // Discard and recycle
        pool.discard_one(TestItem(100));
        pool.recycle_discarded();
        assert_eq!(pool.pool().len(), 7);
    }

    #[test]
    fn test_pool_trait_object_safety() {
        // This test verifies that our traits can be used properly
        let mut pool = TestPool {
            items: vec![TestItem(1), TestItem(2)],
        };

        let item_ref: &mut dyn ItemPool<TestItem> = &mut pool;
        item_ref.put(TestItem(3));
        assert_eq!(item_ref.pool().len(), 3);
    }
}
