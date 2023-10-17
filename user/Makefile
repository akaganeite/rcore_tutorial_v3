TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)#wildcard找到APPDIR下的所有rs文件
#格式：$(patsubst <pattern>,<replacement>,<text> ) 
#名称：模式字符串替换函数——patsubst。
#功能：查找<text>中的单词（单词以“空格”、“Tab”或“回车”“换行”分隔）是否符合模式<pattern>，如果匹配的话，则以<replacement>替换。
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))#target/riscv64gc-unknown-none-elf/release/ch3_sleep

BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

elf: $(APPS)
	@cargo build $(MODE_ARG)

binary: elf
	@echo $(ELFS)
	@$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)
#rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin
build: binary

clean:
	@cargo clean

.PHONY: elf binary build clean