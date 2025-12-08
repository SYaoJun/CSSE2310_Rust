all: uqbasejump
TARGETS := uqbasejump
$(TARGETS):
	cargo build -p a1_2025_s2 --bin a1_2025_s2
	cp ../target/debug/a1_2025_s2 ./uqbasejump
.PHONY: clean
clean:
	rm -f $(TARGETS)