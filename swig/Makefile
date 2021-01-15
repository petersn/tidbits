
CXXFLAGS=-Ofast -g -fPIC -Wall -Wextra `pkg-config --cflags --libs python3` -std=c++17

all: _example.so

%.o: %.cxx
	$(CXX) -c $(CXXFLAGS) -o $@ $^

example_wrap.cxx: example.h example.i
	swig -c++ -python example.i

_example.so: example.o example_wrap.o
	$(CXX) -shared -Wl,-soname,$@ $(CXXFLAGS) -o $@ $^

.PHONY: clean
clean:
	rm -f *.o example_wrap.cxx

