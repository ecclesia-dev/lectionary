PREFIX = /usr/local

lectionary: lectionary.sh data lectionary-cal
	cat lectionary.sh > $@
	echo 'exit 0' >> $@
	echo '#EOF' >> $@
	tar czf - data >> $@
	chmod +x $@

lectionary-cal: cal-helper/src/main.rs cal-helper/Cargo.toml
	cd cal-helper && cargo build --release
	cp cal-helper/target/release/lectionary-cal .

data: build-data.sh
	sh build-data.sh

test: lectionary.sh
	shellcheck -s sh lectionary.sh

clean:
	rm -f lectionary lectionary-cal
	rm -rf data

install: lectionary lectionary-cal
	mkdir -p $(DESTDIR)$(PREFIX)/bin
	cp -f lectionary $(DESTDIR)$(PREFIX)/bin
	cp -f lectionary-cal $(DESTDIR)$(PREFIX)/bin
	chmod 755 $(DESTDIR)$(PREFIX)/bin/lectionary
	chmod 755 $(DESTDIR)$(PREFIX)/bin/lectionary-cal

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/lectionary
	rm -f $(DESTDIR)$(PREFIX)/bin/lectionary-cal

.PHONY: test clean install uninstall
