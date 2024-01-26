#[test]
fn new_owner() {
    fn my_component(
        a: Signal<i32>,
        b: Signal<HashMap<i32, String>>,
        c: Signal<Box<dyn Fn() -> i32>>,
        d: ReadOnlSignal<i32>,
        e: ReadOnlSignal<String>,
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
        let owner = UnsyncSignalStorage::owner();

        let out = owner.insert(Box::new(TrackedSource(123_i32)));

        *out.write() = Box::new(TrackedSource(456)) as Box<dyn SignalSource>;

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

    1
}
