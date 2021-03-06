TARGET_FILES += $(TARGET_DIR)/bin/init
BUILD_DIRS += init/target
INITRAMFS_FILES += /bin/init
FMT_DIRS += init

$(TARGET_DIR)/bin/init: target/$(BUILD_TARGET)/$(BUILD_TYPE)/init
	@mkdir -p $(shell dirname $@)
	cp $< $@

target/$(BUILD_TARGET)/$(BUILD_TYPE)/init: target/$(BUILD_TARGET)/$(BUILD_TYPE)/libinit.a
	$(LINKER) $(LINKER_FLAGS) $< -o $@

target/$(BUILD_TARGET)/$(BUILD_TYPE)/libinit.a: $(shell find init/src -name "*.rs") init/Cargo.toml $(STD_FILES)
	cd init && $(RUST_COMPILER) build $(RUST_COMPILER_FLAGS)
