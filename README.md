# itempool

[![Crates.io](https://img.shields.io/crates/v/itempool.svg)](https://crates.io/crates/itempool)
[![Docs.rs](https://docs.rs/itempool/badge.svg)](https://docs.rs/itempool)
[![License](https://img.shields.io/crates/l/itempool.svg)](./LICENSE)

A lightweight Rust library for managing pools of reusable items with support for random selection, unique set retrieval, and item recycling.

## Features

* **Random Item Selection**: Efficiently retrieve random items from a pool with `remove_one()`
* **Unique Sets**: Get sets of unique items without duplicates using `remove_set()`
* **Item Recycling**: Temporarily discard items and recycle them back into the pool
* **Type Safety**: Uses trait bounds ensuring items are `Hash + Eq + Sync`
* **Zero-cost Abstractions**: Trait-based design with minimal overhead
* **WebAssembly Support**: Works with `wasm32-unknown-unknown` target

## Installation

Add `itempool` to your `Cargo.toml`:

```toml
[dependencies]
itempool = "0.1"
```

## Core Concepts

### `PoolItem` Trait

Items in the pool must implement `PoolItem`, which requires:

* **`Hash`**: For use in hash sets and uniqueness checks
* **`Eq`**: For equality comparisons
* **`Sync`**: For safe concurrent access across threads

### `ItemPool<T>` Trait

The main trait for managing a pool of items. Provides:

* `pool(&mut self) -> &mut Vec<T>`: Access to the main item storage
* `put(&mut self, item: T)`: Add an item to the pool
* `remove_one(&mut self) -> Option<T>`: Remove and return one random item
* `remove_many(&mut self, size: usize) -> Vec<T>`: Remove multiple items (duplicates possible)
* `remove_set(&mut self, size: usize) -> HashSet<T>`: Remove a set of unique items

### `RecyclingItemPool<T>` Trait

Extends `ItemPool` with discard/recycle functionality:

* `get_discard_pool(&mut self) -> &mut Vec<T>`: Access to the discard storage
* `discard_one(&mut self, item: T)`: Move an item to the discard pool
* `recycle_discarded(&mut self)`: Move all discarded items back to the main pool
* `get_set(&mut self, size: usize) -> HashSet<T>`: Get unique items, auto-recycling first

## Example

```rust
use itempool::{ItemPool, RecyclingItemPool};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EntityId(u32);

struct EnemyPool {
    available: Vec<EntityId>,
    discarded: Vec<EntityId>,
}

impl ItemPool<EntityId> for EnemyPool {
    fn pool(&mut self) -> &mut Vec<EntityId> {
        &mut self.available
    }
}

impl RecyclingItemPool<EntityId> for EnemyPool {
    fn get_discard_pool(&mut self) -> &mut Vec<EntityId> {
        &mut self.discarded
    }
}

fn main() {
    let mut pool = EnemyPool {
        available: (0..10).map(EntityId).collect(),
        discarded: vec![],
    };

    // Get a single random enemy
    if let Some(enemy) = pool.remove_one() {
        println!("Spawned enemy: {:?}", enemy);
    }

    // Get a squad of 3 unique enemies
    let squad = pool.remove_set(3);
    println!("Enemy squad: {:?}", squad);

    // Discard an enemy temporarily
    pool.discard_one(EntityId(5));

    // Get more enemies (automatically recycles discarded ones)
    let reinforcements = pool.get_set(2);
    println!("Reinforcements: {:?}", reinforcements);
}
```

See the [examples](examples/) directory for more detailed usage.

## Use Cases

* **Game Development**: Managing enemy spawns, item drops, card decks
* **Resource Pooling**: Reusable connection pools, object pools
* **Random Selection**: Lottery systems, random sampling without replacement
* **State Management**: Temporarily unavailable resources that can be restored

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests on [GitHub](https://github.com/caniko/gpool).

## License

Dual-licensed under MIT or Apache License 2.0.
