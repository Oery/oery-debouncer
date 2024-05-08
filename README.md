# Oery Debouncer

This utility adds a debounce time to your keyboard, this can fix keys with double inputs. Delays are independant to each key. Every time you press a key, you won't be able to press it again for the debounce time.

## Usage

1.  Download the latest release from the [releases page](https://github.com/oery/oery-debouncer/releases)
2.  Execute the executable
3.  Set the debounce time in the system tray icon
4.  Add a link to the executable in your `shell:startup` folder if you want it to start on boot

### Known issues

-   The FN modifier key isn't debounced (Windows API limitation)

## Building

You will need to have the [Rust toolchain](https://www.rust-lang.org/tools/install) installed.

```sh
cargo build --release
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
