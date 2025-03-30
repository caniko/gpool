use item_pool::ItemPool;

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

// Implement the trait for your pool struct
impl ItemPool<EntityId> for EnemyPool {
    fn label(&self) -> String {
        "EnemyPool".to_string()
    }

    fn pool(&mut self) -> &mut Vec<EntityId> {
        &mut self.available_enemies
    }

    fn get_discard_pool(&mut self) -> &mut Vec<EntityId> {
        &mut self.temporarily_disabled
    }

    // Optional: Override discard_one if you need different behavior
    // fn discard_one(&mut self, item: EntityId) {
    //     // Example: Remove from main pool AND add to discard pool
    //     if let Some(pos) = self.available_enemies.iter().position(|&x| x == item) {
    //         self.available_enemies.remove(pos);
    //     }
    //     self.temporarily_disabled.push(item);
    // }
}

fn main() {
    let mut enemy_pool = EnemyPool::new(10); // Pool contains EntityId(0) to EntityId(9)
    println!("Initial Pool: {:?}", enemy_pool.pool()); // [EntityId(0), ..., EntityId(9)]
    println!("Initial Discard: {:?}", enemy_pool.get_discard_pool()); // []
    println!("---");

    // Get a single enemy
    let enemy1 = enemy_pool.get_one();
    println!("Got one enemy: {:?}", enemy1);
    println!("Pool after get_one: {:?}", enemy_pool.pool());
    println!("Discard after get_one: {:?}", enemy_pool.get_discard_pool());
    println!("---");


    // Get a set of enemies
    let enemy_squad = enemy_pool.get_set(3);
    println!("Got enemy squad: {:?}", enemy_squad);
    println!("Pool after get_set: {:?}", enemy_pool.pool());
    println!("Discard after get_set: {:?}", enemy_pool.get_discard_pool());
    println!("---");

    // Manually discard an enemy (using default implementation)
    // Let's discard an ID we know might still be in the pool, e.g., EntityId(0)
    // Or one that might have been removed - the default discard doesn't care.
    let enemy_to_disable = EntityId(5); // This might or might not be present in pool() anymore
    enemy_pool.discard_one(enemy_to_disable);
    println!("Discarded enemy ID: {:?}", enemy_to_disable);
    println!("Pool after discard_one: {:?}", enemy_pool.pool()); // Unchanged by default discard
    println!("Discard after discard_one: {:?}", enemy_pool.get_discard_pool()); // Contains EntityId(5)
    println!("---");


    // Get another enemy - this will trigger recycling first
    println!("Calling get_one, which will recycle first...");
    let enemy2 = enemy_pool.get_one();
    println!("Got another enemy: {:?}", enemy2); // Could be EntityId(5) or another
    println!("Pool after recycle + get_one: {:?}", enemy_pool.pool());
    println!("Discard after recycle + get_one: {:?}", enemy_pool.get_discard_pool()); // Empty now
    println!("---");

    // Recycle remaining discarded items manually (none left here, but demonstrates call)
    enemy_pool.recycle_discarded();
    println!("Pool after manual recycle: {:?}", enemy_pool.pool());
    println!("Discard after manual recycle: {:?}", enemy_pool.get_discard_pool());
}
