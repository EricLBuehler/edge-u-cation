# Hardcoded path to your Android NDK installation
ANDROID_NDK_ROOT = /Users/ericbuehler/Library/Android/sdk/ndk/28.0.12916984

# Target ABI (used for the build command) and the directory containing all jniLibs subdirectories
TARGET = arm64-v8a
JNI_LIBS_DIR = ../main/jniLibs

.PHONY: all build copy_lib clean

# Default target: build the project and then copy the shared library to every subfolder
all: build copy_lib

build:
	@echo "Building release binary for $(TARGET)..."
	# Set the ANDROID_NDK_ROOT only for this command.
	ANDROID_NDK_ROOT=$(ANDROID_NDK_ROOT) cargo +nightly ndk -t $(TARGET) -o $(JNI_LIBS_DIR) build --release

copy_lib:
	@echo "Copying libc++_shared.so to all subdirectories in $(JNI_LIBS_DIR)..."
	@for dir in $(JNI_LIBS_DIR)/*; do \
		if [ -d "$$dir" ]; then \
			echo "Copying to $$dir"; \
			cp $(ANDROID_NDK_ROOT)/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so $$dir; \
		fi; \
	done

clean:
	@echo "Cleaning project..."
	cargo clean
