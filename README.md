# i3daemon

(i3wm)[https://i3wm.org/] event listening daemon for automatic workspace renaming based on opened windows.

Build:

```bash
$ cargo build --release
$ cp target/release/i3daemon ~/bin
```

Example systemd service file:

```
[Unit]
Description=i3daemon

[Service]
Environment=DISPLAY=:0
ExecStart=%h/bin/i3daemon
Restart=on-failure
RestartSec=5s

StandardOutput=append:/tmp/i3daemon.log
StandardError=append:/tmp/i3daemon.log

[Install]
WantedBy=default.target
```
