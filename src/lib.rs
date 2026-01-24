//! Provides a token type [`LocalScope`] that guards access to thread local storage. Makes it easier to work with thread locals inside the scope.
//!
//! # Examples
//!
//! You can use the scoping to gracefully handle errors.
//!
//! ```
//! # use thread_local_scope::local_scope;
//! # struct Whatever();
//! # impl Whatever { fn new() -> Self { Self() } }
//! #
//! thread_local! {
//!     static WHATEVER: Whatever = Whatever::new();
//! }
//!
//! fn with_whatever<R>(f: impl FnOnce(&Whatever) -> R) -> R {
//!     local_scope(|scope| {
//!         if let Ok(x) = scope.try_access(&WHATEVER) {
//!             f(x)
//!         } else {
//!             let stack_local_fallback = Whatever::new();
//!             f(&stack_local_fallback)
//!         }
//!     })
//! }
//!
//! // The equivalent without this requires .unwrap()
//! fn with_whatever_std<R>(f: impl FnOnce(&Whatever) -> R) -> R {
//!     let mut f = Some(f);
//!     WHATEVER
//!         .try_with(|x| f.take().unwrap()(x))
//!         .unwrap_or_else(|_| {
//!             let stack_local_fallback = Whatever::new();
//!             f.unwrap()(&stack_local_fallback)
//!         })
//! }
//! ```
//!
//!
//!
//! This allows avoiding nested closures if working with multiple LocalScopes.
//! ```
//! # use std::{thread::LocalKey, cell::Cell};
//! # use thread_local_scope::local_scope;
//!
//! fn swap_local_cells<T>(a: &'static LocalKey<Cell<T>>, b: &'static LocalKey<Cell<T>>) {
//!     local_scope(|s| {
//!         s.access(a).swap(s.access(b))
//!     })
//! }
//!
//! fn swap_local_cells_std<T>(a: &'static LocalKey<Cell<T>>, b: &'static LocalKey<Cell<T>>) {
//!     a.with(|a| b.with(|b| a.swap(b)))
//! }
//! ```

use std::{
    fmt,
    marker::PhantomData,
    thread::{AccessError, LocalKey},
};

/// ZST token that guarantees consistent access to thread local storage values for the duration of `'a`.
///
/// Created with [`local_scope`].
///
/// # Thread safety
///
/// This marker is locked on the thread that it is created on.
#[derive(Clone, Copy)]
pub struct LocalScope<'a>(PhantomData<*const &'a ()>);

static_assertions::assert_not_impl_any!(LocalScope<'static>: Send, Sync);
static_assertions::assert_eq_size!(LocalScope<'static>, ());

impl<'a> fmt::Debug for LocalScope<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThreadLocalScope").finish()
    }
}

/// Crates a new [`LocalScope`] bound to the current call stack.
///
/// Within the callback function, thread local storage can be freely accessed
pub fn local_scope<F, R>(f: F) -> R
where
    F: for<'a> FnOnce(LocalScope<'a>) -> R,
{
    f(
        // Safety, because 'a is unbound in the callback signature, this lifetime is limited to the duration of this call, during which we can't enter any TLS destructors
        unsafe { LocalScope::unguarded_new() },
    )
}

impl<'a> LocalScope<'a> {
    /// Creates an unguarded [LocalScope].
    ///
    /// # Safety
    ///
    /// The current thread's TLS must live for at least `'a` and no TLS destructors may be entered during `'a`.
    pub const unsafe fn unguarded_new() -> Self {
        Self(PhantomData)
    }

    /// Equivalent to [`LocalKey::try_with`] without the need for the closure.
    pub fn try_access<T>(self, target: &'static LocalKey<T>) -> Result<&'a T, AccessError> {
        target.try_with(
            #[inline]
            |tls| {
                // safety: tls is a reference to data that lives in a TLS. by the condition on Self, this reference must actually live for 'a
                unsafe { &*(tls as *const T) }
            },
        )
    }

    /// Equivalent to [`LocalKey::with`] without the need for the closure.
    pub fn access<T>(self, target: &'static LocalKey<T>) -> &'a T {
        match self.try_access(target) {
            Ok(x) => x,
            Err(ae) => panic_access_error(ae),
        }
    }
}

#[cfg_attr(not(panic = "immediate-abort"), inline(never))]
#[track_caller]
#[cold]
fn panic_access_error(err: AccessError) -> ! {
    panic!("cannot access a Thread Local Storage value during or after destruction: {err:?}")
}

#[cfg(test)]
mod test {
    use crate::*;
    use std::{
        cell::Cell,
        sync::atomic::{AtomicUsize, Ordering},
        thread::spawn,
    };

    #[test]
    fn re_entrant() {
        static DID_RUN_DESTRUCTOR: AtomicUsize = AtomicUsize::new(0);

        thread_local! {
            static MY_THING: MyThing = MyThing;
        }

        struct MyThing;
        impl Drop for MyThing {
            fn drop(&mut self) {
                local_scope(|sc| {
                    // we don't care, since we join to sync with the main thread anyways
                    DID_RUN_DESTRUCTOR.fetch_add(1, Ordering::Relaxed);
                    assert!(
                        sc.try_access(&MY_THING).is_err(),
                        "Can't access self while in destructor"
                    )
                })
            }
        }

        spawn(|| {
            local_scope(|s| {
                let _ = s.try_access(&MY_THING).expect("Testing, should be defined");
            })
        })
        .join()
        .unwrap();

        assert_eq!(DID_RUN_DESTRUCTOR.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn swap() {
        fn swap_local_cells<T>(a: &'static LocalKey<Cell<T>>, b: &'static LocalKey<Cell<T>>) {
            local_scope(|s| s.access(a).swap(s.access(b)))
        }

        thread_local! {
            static A: Cell<u8> = Cell::new(0);
            static B: Cell<u8> = Cell::new(1);
        }

        swap_local_cells(&A, &B);

        assert_eq!(A.get(), 1);
        assert_eq!(B.get(), 0);
    }
}
