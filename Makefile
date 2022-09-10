prog :=packet_monitor

debug ?=

$(info debug is $(debug))

ifdef debug
  release :=
  target :=debug
  extension :=debug
else
  release :=--release
  target :=release
  extension :=
endif

.PHONY: default
default: all ;

build:
	cargo build $(release)

move:
	cp target/$(target)/$(prog) ./proja

clean:
	cargo clean
	rm ./proja

all: build move
 
help:
	@echo "usage: make [debug=1]"