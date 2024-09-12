# denovo-audire

**denovo-audire** is a simple terminal-based music player written in Rust. It allows you to play local `.mp3` files with basic navigation and queue functionality. It's lightweight and easy to use, with controls designed for intuitive playback from the terminal.

## Features

- **Play local `.mp3` files**: Only `.mp3` format is supported for now.
- **Simple controls**: Navigate and control playback easily using your keyboard.
- **Queue management**: Add songs to a queue and play them sequentially.

## Controls

- `Enter`: Play the selected song once.
- `Up/Down Arrow and K/J Keys`: Navigate through the list of songs.
- `Left/Right Arrow and H/L Keys`: Adjust the volume.
- `Q`: Add the selected song to the queue.
- `P`: Play the queued songs.
- `X`: Clear the queue.

## Getting Started

### Precompiled Executable (Windows only)

You can download the precompiled `.exe` from the [Releases](https://github.com/your-username/denovo-audire/releases) page. No additional dependencies are required to run the `.exe`.

### Running from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/denovo-audire.git
   cd denovo-audire

2. Build the project:
   ```bash
   cargo build --release

3. Run the player:
   ```bash
   cargo run --release
   or
   ./target/release/denovo-audire.exe

### Cross-Platform Compatibility

While the player should work cross-platform, it has only been tested on Windows. If you encounter any issues on other platforms, feel free to open an issue.

## Contributing

I'm open to contributions! However, I'm new to managing contributions, so any help in improving both the player and contribution process is welcome.

1. Fork the repository.
2. Create a new branch for your feature/bugfix.
3. Submit a pull request.

Please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
