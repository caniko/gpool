# item_pool

[![Crates.io](https://img.shields.io/crates/v/item_pool.svg)](https://crates.io/crates/item_pool) [![Docs.rs](https://docs.rs/item_pool/badge.svg)](https://docs.rs/item_pool) [![Build Status](https://github.com/YOUR_USERNAME/item_pool/actions/workflows/rust.yml/badge.svg)](https://github.com/YOUR_USERNAME/item_pool/actions) [![License](https://img.shields.io/crates/l/item_pool.svg)](./LICENSE) A small Rust library providing a trait (`ItemPool`) for managing pools of reusable items. It allows for efficient random retrieval of single items or unique sets of items, with support for temporarily discarding and recycling items.

## Features

* **`ItemPool<T>` Trait:** Define the core logic for managing an item pool and a discard pool.
* **Random Retrieval:** Get a single random item (`get_one`) or a specified number of unique random items (`get_set`).
* **Discard & Recycle:** Temporarily move items to a discard pool (`discard_one`) and later move them all back to the main pool (`recycle_discarded`). `get_one` and `get_set` automatically recycle discarded items before drawing.
* **Type Safety:** Uses generics (`T: PoolItem`) where `PoolItem` requires `Copy + Hash + Eq + Sync`.
* **Bevy Integration (Optional):** Includes a `bevy` feature flag for integrating item pools as Bevy resources (requires enabling the feature).

## Installation

Add `item_pool` to your `Cargo.toml`:

```toml
[dependencies]
item_pool = "0.1.0" # Use the latest version from crates.io
```

If you need Bevy integration, enable the bevy feature:

```toml
[dependencies]
item_pool = { version = "0.1.0", features = ["bevy"] }
```

## Core Concepts

`PoolItem` is a type alias for traits required by items stored in the pool: `std::hash::Hash + Eq + Sync`. This ensures items can be:

* **Hashed:** Used in hash sets and hash maps.
* **Compared for Equality:** Checked if two items are the same.
* **Safely Shared:** Accessed by multiple threads concurrently.

`ItemPool<T>` is the main trait you'll implement. It defines the interface for an object that manages a pool of items of type `T`.

* `label(&self) -> String`: Returns a descriptive name for the pool (used in debug assertions).
* `pool(&mut self) -> &mut Vec<T>`: Provides mutable access to the main vector containing available items.
* `get_discard_pool(&mut self) -> &mut Vec<T>`: Provides mutable access to a secondary vector holding items that have been temporarily discarded.
* `discard_one(&mut self, item: T)`: Adds an item to the discard pool.
  * **Note:** The default implementation only adds to the discard pool; it does not remove the item from the main pool. Implementations might override this or handle removal separately if needed.
* `recycle_discarded(&mut self)`: Moves all items from the discard pool back into the main pool.
* `get_one(&mut self) -> T`: Recycles discarded items, then removes and returns one randomly selected item from the main pool. Panics if the pool is empty after recycling.
* `get_set(&mut self, size: usize) -> HashSet<T>`: Recycles discarded items, then removes and returns a `HashSet` containing `size` unique, randomly selected items from the main pool. Panics if the pool is empty after recycling. Handles internal collisions if the same item is randomly drawn multiple times during the process.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests on the repository.

## License

Dual license MIT or Apache License v2