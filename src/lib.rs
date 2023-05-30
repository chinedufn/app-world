//! app-world provides a framework agnostic approach to managing frontend application state.
//!
//! # The Data Model
//!
//! An `AppWorld` is a type that you define that holds your application state as well as other
//! resources that you've deemed useful to have around during your application's runtime.
//!
//! Here's an example of what an AppWorld for a basic e-commerce app frontend might look like:
//!
//! ```rust
//! # use std::collections::HashMap;
//! struct MyAppWorld {
//!     state: MyAppState,
//!     resources: MyAppResources
//! }
//!
//! struct MyAppState {
//!     user: User,
//!     products: HashMap<Uuid, Product>
//! }
//!
//! struct MyAppResources {
//!     file_store: Box<dyn MyFileStoreTrait>,
//!     api_client: ApiClient
//! }
//!
//! # trait MyFileStoreTrait {}
//! # type ApiClient = ();
//! # type Product = ();
//! # type User = ();
//! # type Uuid = ();
//! ```
//!
//! The `MyAppWorld` struct would be defined in your crate, but it wouldn't be used directly when
//! you were passing data around to your views.
//!
//! Instead, you wrap it in an `app_world::AppWorldWrapper<W>`
//!
//! ```rust
//! type MyAppWorldWrapper = app_world::AppWorldWrapper<MyAppWorld>;
//!
//! # type MyAppWorld = ();
//! ```
//!
//! # AppWorldWrapper<W: AppWorld>
//!
//! The `AppWorldWrapper` prevents direct mutable access to your application state, so you cannot
//! mutate fields wherever you please.
//!
//! Instead, the [`AppWorld`] trait defines a [`AppWorld.msg()`] method that can be used to update
//! your application state.
//!
//! You can pass your `AppWorldWrapper<W>` to different threads by calling
//! [`AppWorldWrapper.clone()`]. Under the hood an [`Arc`] is used to share your data across
//! threads.
//!
//! # Example Usage
//!
//! TODO
//!
//! # When to Use app-world
//!
//! app-world shines in applications that do not have extreme real time rendering requirements,
//! such as almost all browser, desktop and mobile applications.
//! In games and real-time simulations, you're better off using something like an entity component
//! system to manage your application state.
//!
//! This is because app-world is designed such that your application state can only be written to
//! from one thread at a time. This is totally fine for almost all browser, desktop and mobile
//! applications, but could be an issue for games and simulations.
//!
//! If you're writing a game or simulation you're likely better off reaching for an
//! entity-component-system library. Otherwise, you should be in good hands here.
//! which could be an issue for a high-performing game or simulation.

#![deny(missing_docs)]

use std::cell::RefCell;
use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread::LocalKey;

/// Holds application state and resources.
/// See the [crate level documentation](crate) for more details.
///
/// # Cloning
///
/// Cloning an `AppWorldWrapper` is a very cheap operation.
///
/// All clones hold pointers to the same inner state.
pub struct AppWorldWrapper<W: AppWorld> {
    world: Arc<RwLock<W>>,
}

/// Defines how messages that indicate that something has happened get sent to the World.
pub trait AppWorld: Sized {
    /// Indicates that something has happened.
    ///
    /// ```
    /// # use std::time::SystemTime;
    /// #[allow(unused)]
    /// enum MyMessageType {
    ///     IncreaseClickCounter,
    ///     SetLastPausedAt(SystemTime)
    /// }
    /// ```
    type Message;

    /// Send a message to the state object.
    /// This will usually lead to a state update
    fn msg(&mut self, message: Self::Message);
}

impl<W: AppWorld + 'static> AppWorldWrapper<W> {
    /// Create a new AppWorldWrapper.
    pub fn new(world: W) -> Self {
        let world = Arc::new(RwLock::new(world));
        Self { world }
    }

    /// Acquire write access to the AppWorld then send a message.
    pub fn msg(&self, msg: W::Message) {
        self.world.write().unwrap().msg(msg)
    }
}

impl<W: AppWorld + 'static> AppWorldWrapper<W> {
    thread_local!(
        static HAS_READ: RefCell<bool> = RefCell::new(false);
    );

    /// Acquire read access to AppWorld.
    ///
    /// # Panics
    /// Panics if the current thread is already holding a read guard.
    ///
    /// This panic prevents the following scenario from deadlocking:
    ///
    /// 1. Thread A acquires a read guard
    /// 2. Thread B calls `AppWorld::msg`, which attempts to acquire a write lock
    /// 3. Thread A attempts to acquire a second read guard while the first is still active
    pub fn read(&self) -> WorldReadGuard<'_, W> {
        Self::HAS_READ.with(|has_read| {
            let mut has_read = has_read.borrow_mut();

            if *has_read {
                panic!("Thread already holds read guard")
            }

            *has_read = true
        });
        WorldReadGuard {
            guard: self.world.read().unwrap(),
            read_tracker: &Self::HAS_READ,
        }
    }

    /// Acquire write access to AppWorld.
    ///
    /// Under normal circumstances you should only ever write to the world through the `.msg()`
    /// method.
    ///
    /// This .write() method is useful when writing tests where you want to quickly set up some
    /// initial state.
    #[cfg(feature = "test-utils")]
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, W> {
        self.world.write().unwrap()
    }
}

impl<W: AppWorld> Clone for AppWorldWrapper<W> {
    fn clone(&self) -> Self {
        AppWorldWrapper {
            world: self.world.clone(),
        }
    }
}

/// Holds a read guard on a World.
pub struct WorldReadGuard<'a, W> {
    guard: RwLockReadGuard<'a, W>,
    read_tracker: &'static LocalKey<RefCell<bool>>,
}
impl<'a, W> Deref for WorldReadGuard<'a, W> {
    type Target = RwLockReadGuard<'a, W>;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}
impl<'a, W> Drop for WorldReadGuard<'a, W> {
    fn drop(&mut self) {
        self.read_tracker.with(|has_reads| {
            *has_reads.borrow_mut() = false;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// Verify that we prevent deadlocks when a thread tries to acquire a read guard on the world
    /// twice.
    ///
    /// ---
    ///
    /// Given Thread A and B, a deadlock can occur if:
    ///
    /// 1. Thread A acquires read guard
    /// 2. Thread B begins waiting for a write guard
    /// 3. Thread A begins waiting for a second read guard
    ///
    /// On some platforms the guard acquisition order would be Thread A, Thread A, Thread B.
    /// That is to say that attempts to acquire write guards do not block attempts to acquire read
    /// guards.
    ///
    /// On other platforms, attempts to write may take precedence over attempts to read.
    ///
    /// On those platforms, Thread A will deadlock on the second read, and Thread B will deadlock
    /// on the write.
    ///
    /// On macOS Ventura the sequence described above will cause a deadlock.
    ///
    /// This test uses two threads and `std::time::sleep` to simulate the sequence above and
    /// confirm that we panic if a thread tries to hold two active read guards at once.
    #[test]
    #[should_panic = "Second read attempt panicked"]
    fn deadlock_prevention_same_thread_double_read_another_thread_write() {
        let world = AppWorldWrapper::new(TestWorld { was_mutated: false });
        let world_clone1 = world.clone();
        let world_clone2 = world.clone();

        let handle = thread::spawn(move || {
            let guard_1 = world.read();
            assert_eq!(guard_1.was_mutated, false);

            let handle = thread::spawn(move || {
                world_clone1.msg(());
            });

            thread::sleep(Duration::from_millis(50));
            let guard_3 = world.read();
            assert_eq!(guard_3.was_mutated, true);

            handle.join().unwrap();
        });

        let join = handle.join();

        assert_eq!(world_clone2.read().was_mutated, true);
        join.expect("Second read attempt panicked");
    }

    /// Verify that the same thread can acquire a second read guard after the first has been
    /// dropped.
    #[test]
    fn two_non_colliding_reads() {
        let world = AppWorldWrapper::new(TestWorld::default());

        {
            let _guard = world.read();
        }

        let _guard = world.read();
    }

    #[derive(Default)]
    struct TestWorld {
        was_mutated: bool,
    }
    impl AppWorld for TestWorld {
        type Message = ();
        fn msg(&mut self, _message: Self::Message) {
            self.was_mutated = true;
        }
    }
}
