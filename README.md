# app-world [![Actions Status](https://github.com/chinedufn/app-world/workflows/test/badge.svg)](https://github.com/chinedufn/app-world/actions) [![docs](https://docs.rs/app-world/badge.svg)](https://docs.rs/app-world)

> A framework agnostic approach to managing frontend application state.

`app-world` is a simple thread-safe storage for application state and resources.

`app-world` has zero dependencies.

## Background

Highly interactive frontend applications need a way to interface with large amounts of application state.

Many frontend frameworks come with state management infrastructure that is coupled to the framework.

`app-world` is designed to be used in any frontend application. This makes it well suited those who want to be able to run
their state related logic across multiple target platforms such as the web, mobile and desktop.

With `app-world` you have a single `World` which holds your application `State`, as well your application's `Resource`s.

`Resource`s are typically used to interface with the outside world, such as to write to a file storage or to make an API request.

You then send `Message`s to your world in order to update application state. This is the only way to mutate application state
(with the exception of fields that have interior mutability).

---

`app-world` should not be used for games. Instead try checking out one of Rust's many entity component system crates.

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
    // ...
}

fn main () {
    let world = AppWorldWrapper::new(MyAppWorld::new());

    world.msg(Msg::IncrementCount);
}
```

## Inspiration

- [The Elm Architecture](https://guide.elm-lang.org/architecture) for the decoupling of views and application state.

- [specs](https://github.com/amethyst/specs) for the `World` `State` and `Resource` names.
