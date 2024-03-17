# Contains commands that use the new cranelift backend for the rust compiler

# Nightly build with cranelift backend, about 30% faster than the default, but only works on nightly and for debug builds
build-cranelift: (dep-cranelift "building with cranelift requires the cranelift backend to be installed")
    RUSTFLAGS="-Zcodegen-backend=cranelift" cargo +nightly build

test-cranelift: (dep-cranelift "testing with cranelift requires the cranelift backend to be installed")
    RUSTFLAGS="-Zcodegen-backend=cranelift" cargo +nightly test
