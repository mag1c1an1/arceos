.PHONY: scp_linux

PORT ?= 2333

scp_linux:
	scp -P $(PORT) $(OUT_BIN) ubuntu@localhost:/home/ubuntu/arceos-intel.bin
