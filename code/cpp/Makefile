# Makefile for the C++23 tutorial decoder.
#
#   make                 build build/decode with g++
#   make CXX=clang++     the same with clang (needs libstdc++ 14 or newer)
#   make clean           remove the build directory
#
# Everything except decode.cpp is a header, so there is exactly one compile
# step. The binary is rebuilt whenever any header changes (they are listed
# as prerequisites via the wildcard below).
#
# C++23 is required for <print>, <expected>, std::views::chunk and the
# monadic std::optional operations; GCC 14+ or Clang 18+ with libstdc++ 14+.

CXX      ?= g++
CXXFLAGS ?= -std=c++23 -O2 -Wall -Wextra -pedantic

HEADERS  := $(wildcard *.hpp)
TARGET   := build/decode

.PHONY: all clean

all: $(TARGET)

$(TARGET): decode.cpp $(HEADERS)
	@mkdir -p build
	$(CXX) $(CXXFLAGS) -o $@ decode.cpp

clean:
	rm -rf build
