build:
	cargo build --release

clean:
	cargo clean

install: build
	i3daemon stop
	cp -f target/release/i3daemon ~/bin
	i3daemon restart
