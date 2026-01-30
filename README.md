# thread_local_scope

[![Rust with MIRI](https://github.com/KyleDavidE/thread_local_scope/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/KyleDavidE/thread_local_scope/actions/workflows/rust.yml)

Provides a token type `LocalScope` that guards access to thread local storage. Makes it easier to work with thread locals inside the scope.


```rust
LOCAL_ONE.try_with(|one| {
    LOCAL_TWO.try_with(|two| {
        ...
    })
})??
```

becomes

```rust
local_scope(|scope| {
    let one = scope.try_access(&LOCAL_ONE)?;
    let two = scope.try_access(&LOCAL_TWO)?;
    ...
})?
```
