# Hypnos

Hypnos is the name of the greek god of sleep... But that's not what this software is (you can't code a god (I think)). 

(Our) Hypnos is a simple idle daemon that you can configure to execute actions after an idle period.

## Installation
To build Hypnos, you need to have Rust and Cargo installed. You can then clone this repository and build the project using Cargo:

```bash
git clone https://github.com/axos-project/hypnos.git
cd hypnos
cargo build --release
```

The resulting binary will be located in the `target/release` directory.

If you want to install it as a package, you can build the package:

```bash
makepkg
```

Or, if you just want to have the binary installed system-wide:

```bash
cp target/release/hypnos /usr/local/bin/
```

## Configuration

Hypnos uses a JSON configuration file to define idle rules. By default, it looks for a configuration file at `$HOME/.config/hypnos/config.json`. You can specify a different configuration file using the `-c` or `--config` command-line option.

Here is an example configuration file:

```json
{
    "enabled": true,
    "rules": [
        {
            "timeout": 5,
            "actions": "brightnessctl -s set 10",
            "restore": "brightnessctl -r",
            "on_battery": true,
            "enabled": true
        },
        {
            "timeout": 120,
            "actions": "loginctl lock-session"
        }
    ]
}
```
- `enabled`: If set to false, Hypnos will not execute any rules.

Each rule in the `rules` array has the following fields:
- `timeout`: The idle time in seconds before the actions are executed.
- `actions`: The command(s) to execute when the timeout is reached.
- `restore`: (Optional) The command(s) to execute when user activity is detected after the actions have been executed.
- `on_battery`: (Optional) If set to true, the rule will only apply when the system is running on battery power.
- `enabled`: (Optional) If set to false, the rule will be ignored.

## Usage

Run Hypnos with the following command:

```bash
hypnos
```

Wow, how difficult was that, right?

You can also specify a custom configuration file:

```bash
hypnos -c /path/to/your/config.json
```

## Logging

If you want to have the details of what is happening, you can run Hypnos with the `RUST_LOG` environment variable set to `info` or `debug`:

```bash
RUST_LOG=debug hypnos # Assuming your binary is in your PATH
```

This will provide detailed logs about the idle notifications and actions being executed.