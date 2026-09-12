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
