DESTDIR =
PREFIX = /usr/local

release: target/release/afwm

target/release/afwm:
	@RUSTFLAGS="-C link-args=-s" cargo build --release
	@echo "Now run 'sudo make install' to install afwm."

clean:
	@cargo clean
	@echo "Cleaning build directory."

install:
	install -m755 -- target/release/afwm "$(DESTDIR)$(PREFIX)/bin/"
	@echo "afwm release has been installed."

uninstall:
	@rm -rfv "$(PREFIX)/bin/afwm"
	@echo "afwm has been uninstalled."

.PHONY: release clean install uninstall
