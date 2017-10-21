
CFLAGS=-O3 -lm -Wall -Wextra

all: square_root

square_root: square_root.o Makefile
	$(CC) $(CFLAGS) -o $@ $<

.PHONY: clean
clean:
	rm -f *.o square_root

