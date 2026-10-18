# Makefile for the C++17 tutorial decoder.
#
#   make            build build/decode
#   make clean      remove the build directory
#
# Everything except decode.cpp is a header, so there is exactly one compile
# step. The binary is rebuilt whenever any header changes (they are listed
# as prerequisites via the wildcard below).

CXX      ?= g++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -pedantic

HEADERS  := $(wildcard *.hpp)
TARGET   := build/decode

.PHONY: all clean

all: $(TARGET)

$(TARGET): decode.cpp $(HEADERS)
	@mkdir -p build
	$(CXX) $(CXXFLAGS) -o $@ decode.cpp

clean:
	rm -rf build
