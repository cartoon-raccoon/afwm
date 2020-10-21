PREFIX = /usr/local
LOG    = cargo_build.log

release:
    # Build release binary
	RUSTFLAGS="-C link-args=-s" cargo build --quiet --release 2> ${LOG}
	@echo "Finished building: release"

debug:
    # Build debug binary
	cargo build --quiet 2> ${LOG}
	@echo "Finished building: debug"

clean:
    # Clean build folder + downloaded deps
	cargo clean --quiet
	@echo "Cleaned build directory"

install: release
	# Ensure destination exists
	mkdir -p ${DESTDIR}${PREFIX}

	# Install binary to location
	install -m755 -- target/release/afwm ${DESTDIR}${PREFIX}/bin/
	@echo "afwm release has been installed"

uninstall:
    # Remove installed binary if present
	rm -f ${PREFIX}/bin/afwm
	@echo "afwm has been uninstalled"

.PHONY: release debug clean install uninstall
