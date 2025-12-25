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
    "rules": {
        "dim": {
            "timeout": 300,
            "actions": "brightnessctl -s set 10",
            "restore": "brightnessctl -r",
            "on_battery": true
        },
        "lock": {
            "timeout": 600,
            "actions": "loginctl lock-session",
            "on_battery": false,
            "enabled": false
        }
    }
}
```
- `enabled`: If set to false, Hypnos will not execute any rules.

Each rule in the `rules` object are defined as follows:
```jsonc
"name": { // Name of the rule, can be anything
    "timeout": <number>, // Time in seconds before the action is executed
    "actions": "<string>", // Command to execute when the timeout is reached
    "restore": "<string>", // (Optional) Command to execute when user activity is detected again
    "on_battery": <boolean>, // (Optional) Whether to execute this rule only when on battery power, defaults to false
    "enabled": <boolean> // (Optional) Whether this rule is enabled, defaults to true
}
```

## Usage

### Daemon mode
This mode allows hypnos to run while the command is running.
Run Hypnos as a daemon with the following command:

```bash
hypnos daemon
```

You can also specify a custom configuration file:

```bash
hypnos daemon -c /path/to/your/config.json
```

 ### Service mode
Hypnos can run as a systemd service.
First, install the service:
```
hypnos install
```
Then, start and enable the service:
```
hypnos enable
hypnos start
```
You can check the status of the service with:
```
systemctl --user status hypnos.service
```
> [!NOTE]
> As hypnos is made for Sleex, the config file should be at `~/.sleex/hypnos.json`.


## Logging

If you want to have the details of what is happening, you can run Hypnos with the `RUST_LOG` environment variable set to `info` or `debug`:

```bash
RUST_LOG=debug hypnos # Assuming your binary is in your PATH
```

This will provide detailed logs about the idle notifications and actions being executed.
