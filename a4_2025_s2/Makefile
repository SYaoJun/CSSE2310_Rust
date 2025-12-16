all:  uqentropy
TARGETS := uqentropy
$(TARGETS): 
	cargo build --debug
	cp target/debug/uqentropy .
.PHONY: clean
clean:
	rm -f $(TARGETS)