# AccessKit Android adapter

This is the Android adapter for [AccessKit](https://accesskit.dev/).

## Prerequisites

- Activities that display UI must derive from the [GameActivity](https://developer.android.com/games/agdk/game-activity) class.
- Content must be drawn into a `View` stored in the `mSurfaceView` field of the `GameActivity`.

## Setup

On the Java side, `Adapter.initialize` must be called before `super.onCreate`.

On the Rust side, `extern crate accesskit_android` must be placed at the root of the crate so that FFI functions gets exported.
