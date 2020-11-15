#!/bin/sh

# Verbose settings
#set -x #echo on
cmd() { echo "\e[96m\$> $@\e[0m" ; "$@" ; }


# Variables

EXE_NAME=sysprog_compiler

ARCH=x86_64

TRIPLE_LINUX=unknown-linux-gnu
TRIPLE_WINDOWS=pc-windows-gnu

TARGET_LINUX=${ARCH}-${TRIPLE_LINUX}
TARGET_WINDOWS=${ARCH}-${TRIPLE_WINDOWS}

RELEASE_LINUX=$(pwd -P)/target/${TARGET_LINUX}/release
RELEASE_WINDOWS=$(pwd -P)/target/${TARGET_WINDOWS}/release


# Build

printf "\n\e[33mCompile for Linux ...\e[0m\n"
cmd cargo build --release --verbose --target=${TARGET_LINUX}

printf "\n\e[33mCompile for Windows ...\e[0m\n"
cmd cargo build --release --verbose --target=${TARGET_WINDOWS}

printf "\n\n\e[33mShow Linux ELF format (GCC or LLVM)\e[0m\n"
cmd readelf --string-dump .comment ${RELEASE_LINUX}/${EXE_NAME}

printf "\n\e[33mShow output files\e[0m\n"
#cmd ls -shl $(pwd -P)/target/x86_64-{unknown-linux,pc-windows}-gnu/release/${EXE_NAME}{,.exe} 2>/dev/null
ls -shl ${RELEASE_LINUX}/${EXE_NAME}
ls -shl ${RELEASE_WINDOWS}/${EXE_NAME}.exe
