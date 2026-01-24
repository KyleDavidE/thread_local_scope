use std::thread;

use thread_local_scope::local_scope;

thread_local! {
    static FOO: u8 = const { 0 };
}

fn main() {
    let _: u8 = local_scope(|sco| {
        thread::scope(move |x| x.spawn(move || *sco.access(&FOO)).join()).unwrap()
    });
}
