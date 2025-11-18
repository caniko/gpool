# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-18

### Added

- Initial release of `itempool`
- `PoolItem` trait for type-safe pool items requiring `Hash + Eq + Sync`
- `ItemPool<T>` trait for managing pools of reusable items
  - `pool()` - Access to main item storage
  - `put()` - Add items to the pool
  - `remove_one()` - Remove and return a single random item
  - `remove_many()` - Remove multiple items (duplicates possible)
  - `remove_set()` - Remove a set of unique items without duplicates
- `RecyclingItemPool<T>` trait extending `ItemPool` with discard/recycle functionality
  - `get_discard_pool()` - Access to discard storage
  - `discard_one()` - Move items to discard pool temporarily
  - `recycle_discarded()` - Return all discarded items to main pool
  - `get_set()` - Get unique items with automatic recycling
- WebAssembly support via `wasm32-unknown-unknown` target
- Comprehensive documentation with examples
- Full test coverage (unit tests and doc tests)
- Example implementation (`examples/enemy_example.rs`)

### Dependencies

- `rand` ^0.9.0 for random item selection
- `getrandom` ^0.3.4 with WASM support

[0.1.0]: https://github.com/caniko/itempool/releases/tag/v0.1.0
