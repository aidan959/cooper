# Cooper

Cooper is a game engine for my final year project in 4th year computer science. It is composed of 3 crates, application, lynch and frost.

## Getting Started

To get started with this project, you'll need to have Rust installed. You can download Rust from the official website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) Vulkan will also need to be installed.
The engine currently requires a dedicated GPU to run the engine.

## Creating an instance of the engine
```rust
const WINDOW_SIZE: WindowSize = (1280., 720.);
fn main() {
    CooperApplication::builder()
        .engine_settings(
            EngineSettings::builder()
                .set_window_name("Cooper")
                .fps_max(512).unwrap()
                .window_size(WINDOW_SIZE)
                .build(),
        )
        .camera(
            Camera::builder()
                .fov_degrees(90.)
                .position(const_vec3!([0.0, 0.0, 0.0]))
                .aspect_ratio_from_window(WINDOW_SIZE)
                .build(),
        )
        .build()
        .run()
```

## Building

To build the project, navigate to the project directory and run:
```sh
cargo build
```
Running
## To run the project, use:

```sh
cargo run
```
## Troubleshooting

If you encounter errors related to undeclared types or modules such as EventsLoop, vk_sync, or ImageDesc, ensure that you have the correct dependencies declared in your Cargo.toml file. You may need to import these types or modules at the top of your Rust files with the use keyword.

## Contributing

Contributions are welcome. Please open an issue to discuss your idea before making a pull request