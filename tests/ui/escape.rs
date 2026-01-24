use thread_local_scope::local_scope;

thread_local! {
    static FOO: u8 = const { 0 };
}

fn main() {
    let out = local_scope(|x| {
        let r1 = x.access(&FOO);
        println!("{r1}");
        r1
    });
    println!("{out}");
}
