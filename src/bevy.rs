use bevy_ecs::system::{Commands, Resource};

use crate::{PoolItem, ItemPool};

/// Trait for a Bevy resource that implements the `ItemPool` trait.
pub trait ItemPoolResource<T: PoolItem> = Resource + ItemPool<T>;

/// Trait for a Bevy resource that implements the `ItemPool` trait with Default.
pub trait ItemPoolResourceDefault<T: PoolItem> = ItemPoolResource<T> + Default;

/// System for initializing any initializing pool
pub fn initialize_default_pool<T: PoolItem, P: ItemPoolResourceDefault<T>>(mut commands: Commands) {
    commands.init_resource::<P>();
}
