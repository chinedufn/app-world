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

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

pub use self::world_with_message_buffer::*;

mod world_with_message_buffer;

/// A function that can trigger a re-render of the application.
///
/// In a browser application this might update the DOM. On iOS this might increment a @Published
/// variable in SwiftUI.
#[cfg(not(feature = "send"))]
pub type RenderFn = Arc<Mutex<Box<dyn FnMut() -> ()>>>;

/// A function that can trigger a re-render of the application.
///
/// In a browser application this might update the DOM. On iOS this might increment a @Published
/// variable in SwiftUI.
#[cfg(feature = "send")]
pub type RenderFn = Arc<Mutex<Box<dyn FnMut() -> () + Send + Sync>>>;

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
pub struct AppWorldWrapper<W: AppWorld> {
    world: Arc<RwLock<WorldWithMessageBuffer<W>>>,
    render_fn: RenderFn,
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

    /// Whether or not the application should be told to re-render.
    /// This check occurs before the messae is processed.
    fn should_rerender(&self, message: &Self::Message) -> bool;
}

impl<W: AppWorld> AppWorldWrapper<W> {
    /// Create a new AppWorldWrapper.
    pub fn new(world: W, render_fn: RenderFn) -> Self {
        Self {
            world: Arc::new(RwLock::new(WorldWithMessageBuffer::new(world))),
            render_fn,
        }
    }

    /// Acquire write access to the AppWorld then send a message.
    pub fn msg(&self, msg: W::Message) {
        let should_rerender = self.world.read().unwrap().should_rerender(&msg);

        self.world
            .write()
            .unwrap()
            .message_maybe_capture(msg, self.clone());

        if should_rerender {
            (self.render_fn.lock().unwrap())();
        }
    }

    /// Acquire read access to AppWorld.
    pub fn read(&self) -> RwLockReadGuard<'_, WorldWithMessageBuffer<W>> {
        self.world.read().unwrap()
    }
}

impl<S: AppWorld> Clone for AppWorldWrapper<S> {
    fn clone(&self) -> Self {
        AppWorldWrapper {
            world: Arc::clone(&self.world),
            render_fn: Arc::clone(&self.render_fn),
        }
    }
}
