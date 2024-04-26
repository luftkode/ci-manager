# Contains commands that use the new cranelift backend for the rust compiler

# Fast build with cranelift backend
fbuild: build-cranelift

# Fast debug run with the fast compile settings
frun *ARGS: fbuild
    ./target/debug/ci-manager {{ ARGS }}

# fast build with cranelift backend, mold linker, and parallel rustc frontend (not necessarily faster, but try it out)
fbuild-alt $RUSTFLAGS="-Zcodegen-backend=cranelift -Zthreads=0": dep-fbuild (dep-cranelift "building with cranelift requires the cranelift backend to be installed")
    RUSTFLAGS="${RUSTFLAGS} -Clinker=$(which clang) -Clink-arg=-fuse-ld=$(which mold)" cargo +nightly build --target x86_64-unknown-linux-gnu

# Nightly build with cranelift backend, about 30% faster than the default, but only works on nightly and for debug builds
[private]
build-cranelift $RUSTFLAGS="-Zcodegen-backend=cranelift": (dep-cranelift "building with cranelift requires the cranelift backend to be installed")
    cargo +nightly build

[private]
test-cranelift: (dep-cranelift "testing with cranelift requires the cranelift backend to be installed")
    RUSTFLAGS="-Zcodegen-backend=cranelift" cargo +nightly test

[private]
dep-fbuild:
    #!/usr/bin/env bash
    declare -i exitval=0
    if ! which mold > /dev/null; then
        {{PRINT}} red "Fast compile requires mold to be installed."
        exitval=1
    fi
    if  ! which clang > /dev/null; then
        {{PRINT}} red "Fast compile requires clang to be installed."
        exitval=1
    fi
    exit $exitval
