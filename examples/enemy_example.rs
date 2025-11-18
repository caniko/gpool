use itempool::{ItemPool, RecyclingItemPool};

// Define your item type (must satisfy PoolItem requirements)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EntityId(u32);

// Create a struct to hold your pool data
#[derive(Debug)]
struct EnemyPool {
    available_enemies: Vec<EntityId>,
    temporarily_disabled: Vec<EntityId>,
}

impl EnemyPool {
    fn new(count: u32) -> Self {
        EnemyPool {
            available_enemies: (0..count).map(EntityId).collect(),
            temporarily_disabled: Vec::new(),
        }
    }
}

// Implement the ItemPool trait for your pool struct
impl ItemPool<EntityId> for EnemyPool {
    fn pool(&mut self) -> &mut Vec<EntityId> {
        &mut self.available_enemies
    }
}

// Implement the RecyclingItemPool trait
impl RecyclingItemPool<EntityId> for EnemyPool {
    fn get_discard_pool(&mut self) -> &mut Vec<EntityId> {
        &mut self.temporarily_disabled
    }
}

fn main() {
    let mut enemy_pool = EnemyPool::new(10); // Pool contains EntityId(0) to EntityId(9)
    println!("Initial Pool: {:?}", enemy_pool.pool()); // [EntityId(0), ..., EntityId(9)]
    println!("Initial Discard: {:?}", enemy_pool.get_discard_pool()); // []
    println!("---");

    // Get a single enemy
    let enemy1 = enemy_pool.remove_one();
    println!("Got one enemy: {:?}", enemy1);
    println!("Pool after remove_one: {:?}", enemy_pool.pool());
    println!(
        "Discard after remove_one: {:?}",
        enemy_pool.get_discard_pool()
    );
    println!("---");

    // Get a set of enemies
    let enemy_squad = enemy_pool.remove_set(3);
    println!("Got enemy squad: {:?}", enemy_squad);
    println!("Pool after remove_set: {:?}", enemy_pool.pool());
    println!(
        "Discard after remove_set: {:?}",
        enemy_pool.get_discard_pool()
    );
    println!("---");

    // Manually discard an enemy
    let enemy_to_disable = EntityId(5);
    enemy_pool.discard_one(enemy_to_disable);
    println!("Discarded enemy ID: {:?}", enemy_to_disable);
    println!("Pool after discard_one: {:?}", enemy_pool.pool());
    println!(
        "Discard after discard_one: {:?}",
        enemy_pool.get_discard_pool()
    ); // Contains EntityId(5)
    println!("---");

    // Get another enemy using get_set - this will trigger recycling first
    println!("Calling get_set, which will recycle first...");
    let enemy2 = enemy_pool.get_set(1);
    println!("Got another enemy: {:?}", enemy2);
    println!("Pool after recycle + get_set: {:?}", enemy_pool.pool());
    println!(
        "Discard after recycle + get_set: {:?}",
        enemy_pool.get_discard_pool()
    ); // Empty now
    println!("---");

    // Demonstrate manual recycling
    enemy_pool.discard_one(EntityId(7));
    enemy_pool.discard_one(EntityId(8));
    println!("Discarded 2 more enemies");
    println!("Pool before recycle: {:?}", enemy_pool.pool());
    println!(
        "Discard before recycle: {:?}",
        enemy_pool.get_discard_pool()
    );

    enemy_pool.recycle_discarded();
    println!("Pool after manual recycle: {:?}", enemy_pool.pool());
    println!(
        "Discard after manual recycle: {:?}",
        enemy_pool.get_discard_pool()
    );
}
