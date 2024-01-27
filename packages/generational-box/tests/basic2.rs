use std::sync::OnceLock;

use generational_box::*;

thread_local! {
    static ARENA: &'static UnsyncArena<i32> = Box::leak(Box::new(Arena::new()));
}

static SYNC_ARENA: OnceLock<SyncArena<i32>> = OnceLock::new();

#[test]
fn using_local() {
    ARENA.with(|a| {
        a.insert(10);
    });

    SYNC_ARENA.get_or_init(|| Arena::new_sync()).insert(10);
}

#[test]
fn new() {
    let arena = Arena::new::<i32>();

    let first = arena.insert(10);

    let o = first.write();
    dbg!(*o);

    let b = arena.insert(123);

    dbg!(b.read());
    drop(o);

    // If none, has alreayd been freed
    let item = arena.free(first);
}

#[test]
fn with_owner() {
    let arena = Arena::new::<i32>();

    let mut owner = arena.owner();

    let first = owner.insert(10);
    let second = owner.insert(10);
    let third = owner.insert(10);
    let fourth = owner.insert(10);

    drop(owner);

    assert!(first.try_read().is_err());
}
