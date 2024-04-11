# Frost

Frost is the entity management system of Cooper.

## Getting Started

To get started with this project, you'll need to have Rust installed. You can download Rust from the official website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

## Creating a World and Inserting into it

```rust
use frost::*;
let mut world = World::new();
world.new_entity((true, "Test String");
let search = world.search<(&bool, &String)>();
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