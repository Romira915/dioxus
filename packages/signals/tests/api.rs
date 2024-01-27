use std::collections::HashMap;

use dioxus_signals::*;

#[test]
fn new_owner() {
    fn my_component(
        a: Signal<i32>,
        b: Signal<HashMap<i32, String>>,
        c: Signal<Box<dyn Fn() -> i32>>,
        d: ReadOnlySignal<i32>,
        e: ReadOnlySignal<String>,
        f: UntrackedSignal<i32>,
        g: UntrackedSignal<i32>,
    ) {
        println!("a: {}", a);
        println!("b: {:?}", b);
        println!("c: {}", c.read()());
        println!("d: {}", d);
        println!("e: {}", e);
        println!("f: {}", f);
        println!("g: {}", g());
    }

    {
        let mut signal: Signal<i32> = Signal::new(123);

        let val = signal.read();
        assert_eq!(*val, 123);
        drop(val);

        let mut val = signal.write();
        *val = 456;
        println!("val: {}", *val);
    }

    let a = Signal::new(123);
    let b = Signal::new(HashMap::new());
    let c = Signal::new(Box::new(move || a()) as _);
    let d = a.clone().into();
    let e = Signal::read_only("hello".to_string());
    let f = Signal::untracked(123);
    let g = Signal::untracked(a()); // there's no way to get a "untracked" variant of a regular signal from a signal - this needs to be fixed

    my_component(a, b, c, d, e, f, g);
}

fn it_works() {
    fn api(a: Signal<i32>, b: ReadOnlySignal<i32>) {
        a.read();
        a.read_untracked();
        a.write();
        a.write_untracked();

        let c = a();
    }

    fn send_signal_is_send(s: Signal<i32, SyncSignalSlot>) {
        fn send<T: Send>(_: T) {}

        send(s);
    }

    fn unsend_signal_is_not_send(s: Signal<Cell<i32>, SyncSignalSlot>) {
        fn send<T: Send>(_: T) {}
        // this should fail!
        // send(s);
    }

    // fn composite(a: Signal<i32>) {
    //     api(a, a.into());
    // }
}
