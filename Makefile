SHELL := /bin/bash
help:
	@echo "tools - build tools"


# 定义根路径
# ROOT_TOOLS_OUTPUT_DIR = .

tools:
	cd $(ROOT_TOOLS_DIR) && cargo build --release
	strip target/release/n-nacos-tools # 去除不必要的符号信息
	# cp -f target/release/nacos-tools $(ROOT_TOOLS_OUTPUT_DIR)/n-nacos-tools


