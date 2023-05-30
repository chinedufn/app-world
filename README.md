# app-world [![Actions Status](https://github.com/chinedufn/app-world/workflows/test/badge.svg)](https://github.com/chinedufn/app-world/actions) [![docs](https://docs.rs/app-world/badge.svg)](https://docs.rs/app-world)

> A framework agnostic approach to managing frontend application state.

## Overview

`app-world` is simple thread-safe state management library that is designed to be useful in cross-platform frontend applications
that manage large amounts of application state.


With `app-world` you have a single `World` which holds your application `State`, as well your application's `Resource`s.

`Resource`s are used to interface with the outside world, such as to write to a local file storage or to make an API request.

The only way to mutate application state is by sending a `Msg` (ignoring `UnsafeCell` based interior mutability).

This means that all state mutation can be handled in a single place, which makes it easy to reason about the application's
behavior and decreases the likelihood of code duplication.

## Cross-Platform Applications

`app-world` does not have any platform dependent code, making it suitable for writing cross-platform application logic that can run on the
web, mobile and desktop.

For example, `app-world` can be used to manage state in a Rust core application that gets run on `iOS`, `Android` and in web browsers.

## Thread Safety

You cannot acquire a write guard on the World directly. The only way to write to a World is via `AppWorld::msg`.

Multiple threads can read from an `AppWorld` simultaneously, but only one `AppWorld::msg` will be processed at a time.

This means that you can safely use `app-world` in multi-threaded applications without worrying about deadlocks.

## Games

Multiple threads can read from an `AppWorld` simultaneously, but only one `AppWorld::msg` will be processed at a time.

This makes `app-world` a poor fit for games that have hardcore performance requirements where you might want many threads to be able to manipulate the World simultaneously.

In those cases, consider using one of the many existing Entity Component System crates.

## Example Usage

```rust
use app_world::AppWorldWrapper;

struct MyAppWorld {
    state: MyAppState,
    resources: MyAppResources,
}

struct MyAppState {
    count: u32
}

struct MyAppResources {
    api_client: Arc<dyn SomeApiClient>
}

enum Msg {
    IncrementCount(u8)
}

type MyAppStateWrapper = AppWorldWrapper<MyAppState>;

impl AppWorld for MyAppWorld {
    type Msg = Msg;

    fn msg(&mut self, message: Msg) {
        match msg {
            Msg::IncrementCount(increment) => {
                self.count += 1;
            }
        }
    }
}

fn main () {
    let world = AppWorldWrapper::new(MyAppWorld::new());
    let world_clone = world.clone();

    assert_eq!(world.read().count, 0);
    world.msg(Msg::IncrementCount);
    world_clone.msg(Msg::IncrementCount);
    assert_eq!(world.read().count, 2);
}
```

## Inspiration

- [The Elm Architecture](https://guide.elm-lang.org/architecture) for the decoupling of views and application state.

- [specs](https://github.com/amethyst/specs) for the `World` `State` and `Resource` names.
