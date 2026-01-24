use thread_local_scope::LocalScope;

fn assert_send<A: Send>() {}
fn assert_sync<A: Sync>() {}

fn main() {
    assert_send::<LocalScope<'static>>();
    assert_sync::<LocalScope<'static>>();
}
