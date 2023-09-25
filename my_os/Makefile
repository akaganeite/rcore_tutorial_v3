kernel:
	cargo build --release
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/my_os -O binary target/riscv64gc-unknown-none-elf/release/my_os.bin


run: kernel
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/my_os.bin,addr=0x80200000 \

debug: kernel
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/rustsbi-qemu.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/my_os.bin,addr=0x80200000 \
    -s -S