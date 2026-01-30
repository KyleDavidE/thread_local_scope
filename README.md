# thread_local_scope

[![Rust with MIRI](https://github.com/KyleDavidE/thread_local_scope/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/KyleDavidE/thread_local_scope/actions/workflows/rust.yml) ![docs.rs](https://img.shields.io/docsrs/thread_local_scope)


Provides a token type `LocalScope` that guards access to thread local storage, avoiding the need for a separate closure for every access.


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
