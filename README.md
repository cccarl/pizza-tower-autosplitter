# Pizza Tower Livesplit One Autosplitter

Main Autosplitter for Pizza Tower using the new autosplitting runtime for LiveSplit One and LiveSplit.

## Features

* 4 Game Time modes for LiveSplit: Full Game, Individual Level, New Game+ and Individual World. Remember to use the launch option "-livesplit" in Pizza Tower for this!
* Customizable start, split and reset events using the new GUI for the autosplitting runtime.
* Tick Rate of 240hz, ASL splitters struggle to keep up with a 60hz tick rate.

## How to use from original LiveSplit

1. Open LiveSplit.
2. Set game name as "Pizza Tower".
3. Click "Activate" button.

## How to manually add release to original LiveSplit:

1. Right Click.
2. Edit Layout...
3. \+ Button -> Control -> Auto Splitting Runtime.
4. Open the added component and look for the WASM file using the file explorer at the top of the window.

## Compilation

This auto splitter is written in Rust. In order to compile it, you need to
install the Rust compiler: [Install Rust](https://www.rust-lang.org/tools/install).

Afterwards install the WebAssembly target:
```sh
rustup target add wasm32-unknown-unknown --toolchain stable
```

The auto splitter can now be compiled:
```sh
cargo b --release
```

The auto splitter is then available at:
```
target/wasm32-unknown-unknown/release/pizza_tower_autosplitter.wasm
```

Make sure too look into the [API documentation](https://livesplit.org/asr/asr/) for the `asr` crate.

## Development

You can use the [debugger](https://github.com/LiveSplit/asr-debugger) while
developing the auto splitter to more easily see the log messages, statistics,
dump memory, step through the code and more.

The repository comes with preconfigured Visual Studio Code tasks. During
development it is recommended to use the `Debug Auto Splitter` launch action to
run the `asr-debugger`. You need to install the `CodeLLDB` extension to run it.

You can then use the `Build Auto Splitter (Debug)` task to manually build the
auto splitter. This will automatically hot reload the auto splitter in the
`asr-debugger`.

Alternatively you can install the [`cargo
watch`](https://github.com/watchexec/cargo-watch?tab=readme-ov-file#install)
subcommand and run the `Watch Auto Splitter` task for it to automatically build
when you save your changes.

The debugger is able to step through the code. You can set breakpoints in VSCode
and it should stop there when the breakpoint is hit. Inspecting variables may
not work all the time.
