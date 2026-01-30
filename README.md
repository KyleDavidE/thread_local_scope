# thread_local_scope

[![Rust with MIRI](https://github.com/KyleDavidE/thread_local_scope/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/KyleDavidE/thread_local_scope/actions/workflows/rust.yml)

Provides a token type `LocalScope` that guards access to thread local storage. Makes it easier to work with thread locals inside the scope.


```rust
WHATEVER.try_with(|r| {
    ...
})
```

becomes

```rust
local_scope(|scope| {
    let r = scope.try_access(&WHATEVER)?
    ...
})
```

Which allows for more flexible code.