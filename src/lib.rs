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
//! # trait MyFileStoreTrait {};
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
//! Whenever you update state using a `.msg()` call, the [`RenderFn`] that you provide is called
//! and your application gets re-rendered.
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

pub use crate::world_with_message_buffer::WorldWithMessageBuffer;
use std::sync::{Arc, RwLock, RwLockReadGuard};

mod world_with_message_buffer;

/// Holds application state and resources and will trigger a re-render after .msg() calls.
/// See the [crate level documentation](crate) for more details.
///
/// # Cloning
///
/// Cloning an `AppWorldWrapper` is a very cheap operation.
///
/// It can be useful to clone `AppWorldWrapper`'s in order to pass the world into event handler
/// closures.
///
/// All clones hold pointers to the same inner state.
pub struct AppWorldWrapper<W: AppWorld + 'static> {
    world: &'static Arc<RwLock<WorldWithMessageBuffer<W>>>,
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
    fn msg(&mut self, message: Self::Message, world_wrapper: AppWorldWrapper<Self>);
}

impl<W: AppWorld + 'static> AppWorldWrapper<W> {
    /// Create a new AppWorldWrapper.
    pub fn new(world: W) -> Self {
        let world = Arc::new(RwLock::new(WorldWithMessageBuffer::new(world)));
        let world = Box::leak(Box::new(world));

        Self { world }
    }

    /// Acquire write access to the AppWorld then send a message.
    pub fn msg(&self, msg: W::Message) {
        self.world
            .write()
            .unwrap()
            .message_maybe_capture(msg, *self);
    }

    /// Acquire read access to AppWorld.
    pub fn read(&self) -> RwLockReadGuard<'_, WorldWithMessageBuffer<W>> {
        self.world.read().unwrap()
    }

    /// Acquire write access to AppWorld.
    ///
    /// Under normal circumstances you should only ever write to the world through the `.msg()`
    /// method.
    ///
    /// This .write() method is useful when writing tests where you want to quickly set up some
    /// initial state.
    #[cfg(feature = "test-utils")]
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, WorldWithMessageBuffer<W>> {
        self.world.write().unwrap()
    }
}

impl<W: AppWorld + 'static> Clone for AppWorldWrapper<W> {
    fn clone(&self) -> Self {
        AppWorldWrapper { world: self.world }
    }
}

impl<W: AppWorld + 'static> Copy for AppWorldWrapper<W> {}
