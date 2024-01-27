use std::rc::Rc;

use dioxus_core::prelude::{has_context, provide_context};
use generational_box::{GenerationalBox, Owner};

use crate::{current_owner, ReadOnly, SignalSource, Source, Writable};

pub struct BaseSignal<T, S, M>
where
    T: 'static,
    S: 'static,
{
    generational_box: S,
    _marker: std::marker::PhantomData<(T, M)>,
}

// pub type ReadOnlySignal<T, S = UnsyncStorage> = Signal<T, S, ReadOnly>;

// impl<T: 'static> Signal<T, S> {
//     pub fn new(value: T) -> Self {
//         let caller = std::panic::Location::caller();
//         let src: Box<dyn Source<T>> = Box::new(SignalSource { value });
//         Signal {
//             generational_box: current_owner().insert_unsized(src, caller),
//             _marker: std::marker::PhantomData,
//         }
//     }
// }

// impl<T: 'static, S: Storage<dyn Source<T>> + 'static, M> Signal<T, S, M> {
//     pub fn read(&self) -> S::Ref<T> {
//         let a = self.generational_box.read();
//         S::map(a, |f| {
//             f.tracked_read();
//             f.read()
//         })
//     }

//     pub fn read_untracked(&self) -> S::Ref<T> {
//         S::map(self.generational_box.read(), |f| f.read())
//     }
// }

fn it_works() {
    fn api(a: Signal<i32>, b: ReadOnlySignal<i32>) {
        a.read();
        // a.write();
        // b.read();
        // let p = a();
        // let q = b();
    }

    // fn composite(a: Signal<i32>) {
    //     api(a, a.into());
    // }
}
