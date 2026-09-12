# Copyright (C) 2026  doclic <doclic@tutanota.com>
#
# Permission to use, copy, modify, and/or distribute this software for any
# purpose with or without fee is hereby granted.
#
# THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
# REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
# AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
# INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
# LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
# OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
# PERFORMANCE OF THIS SOFTWARE.

.POSIX:

# compiler for the machine that runs the build
BUILDCC = $(CC)
BUILDCFLAGS = $(CFLAGS)
BUILDLDFLAGS = $(LDFLAGS)

# compiler for the machine that runs final program
HOSTCC = $(CC)
HOSTCFLAGS = $(CFLAGS)
HOSTLDFLAGS = $(LDFLAGS)

.PHONY: all clean

.DEFAULT: all

all: grainpit.cgi

clean:
	rm -f grainpit.cgi main.o generated.h generate generate.o

grainpit.cgi: main.o
	$(HOSTCC) $(HOSTLDFLAGS) -ograinpit.cgi main.o

main.o: main.c config.h generated.h
	$(HOSTCC) $(HOSTCFLAGS) -c main.c

config.h:
	cp config.def.h config.h

generated.h: generate
	./generate

generate: generate.o
	$(BUILDCC) $(BUILDLDFLAGS) -ogenerate generate.o

generate.o: generate.c
	$(BUILDCC) $(BUILDCFLAGS) -c generate.c
