.PHONY: b00t clean check

b00t:
	cd b00t && cargo build --release
	llvm-objcopy -O binary b00t/target/aarch64-unknown-none-softfloat/release/b00t b00t/target/b00t.img
	python3 tools/pad_b00t.py b00t/target/b00t.img

check:
	cd b00t && cargo fmt --check && cargo clippy --release -- -D warnings
	python3 -m py_compile installer/winm_install.py tools/*.py
	gcc -std=c11 -Wall -Wextra -Werror tests/winm_dma_test.c drivers/common/winm_dma.c drivers/nvme/nvme.c -o /tmp/winm_dma_test
	/tmp/winm_dma_test
	gcc -std=c11 -Wall -Wextra -Werror tests/nvme_core_test.c drivers/nvme/nvme_core.c -o /tmp/nvme_core_test
	/tmp/nvme_core_test

clean:
	rm -rf b00t/target
	find . -name '__pycache__' -type d -prune -exec rm -rf {} +
	find . -name '*.pyc' -delete
