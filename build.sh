#!/bin/bash

# --- Configuration ---
PROJECT_ROOT=$(pwd)
INITRAMFS_DIR="$PROJECT_ROOT/initramfs"
RUST_PROJECT_DIR="$PROJECT_ROOT/symbiote_init"
OUTPUT_INITRD="$PROJECT_ROOT/initrd"
TARGET="x86_64-unknown-linux-musl"

echo "--- 1. Compiling Rust Init (Static, Non-PIE) ---"
cd "$RUST_PROJECT_DIR" || exit

# We force relocation-model=static to avoid the 'Error -8' (PIE) issue
RUSTFLAGS="-C relocation-model=static" cargo build --release --target $TARGET

if [ $? -ne 0 ]; then
    echo "Rust build failed!"
    exit 1
fi

echo "--- 2. Updating Initramfs Layout ---"
# Ensure the binary exists before copying
BINARY_PATH="$RUST_PROJECT_DIR/target/$TARGET/release/symbiote_init"

if [ -f "$BINARY_PATH" ]; then
    cp "$BINARY_PATH" "$INITRAMFS_DIR/init"
    chmod +x "$INITRAMFS_DIR/init"
    echo "[+] Binary copied to /init"
else
    echo "[-] Error: Binary not found at $BINARY_PATH"
    exit 1
fi

echo "--- 3. Packaging Initramfs (CPIO + GZIP) ---"
cd "$INITRAMFS_DIR" || exit

# Find all files, create a newc format cpio archive, and compress it
find . -print0 | cpio --null -ov --format=newc | gzip -9 > "$OUTPUT_INITRD"

if [ $? -eq 0 ]; then
    echo "========================================"
    echo " SUCCESS: $OUTPUT_INITRD is ready."
    echo " You can now run your QEMU command."
    echo "========================================"
else
    echo "Packaging failed!"
    exit 1
fi
cd ..
qemu-system-x86_64 -kernel kernel/linux-7.0.2/arch/x86/boot/bzImage -initrd initrd -append "console=tty0" -cpu host -enable-kvm