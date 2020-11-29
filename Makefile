TMP_DIR=/Volumes/ramdisk
TEST_FILE=$(TMP_DIR)/top-1m-toplevel.txt
NUM_CORES=$(shell nproc --all)
MAX_ITEMS=100

# default target
.PHONY: all
all: benchmark

$(TEST_FILE): top-1m.csv $(TMP_DIR)
	awk -v FS=',' '{n=split($$2, parts, "."); print parts[n]}' $< >| $@

$(TMP_DIR):
	# create a 100MB RAM disk for benchmarking
	diskutil partitionDisk $$(hdiutil attach -nomount ram://204800) 1 GPTFormat APFS 'ramdisk' '100%'

top-1m.csv:
	curl -s -o $@.zip https://s3.amazonaws.com/alexa-static/top-1m.csv.zip
	unzip $@.zip
	rm -f $@.zip

# benchmarking
.PHONY: benchmark
benchmark: bench-bin

bench-unix: $(TEST_FILE)
	hyperfine -m 20 --style basic --warmup 3 "gsort --parallel=$(NUM_CORES) $< | guniq -c | gsort --parallel=$(NUM_CORES) -k1,1 -rn | ghead -n $(MAX_ITEMS)"

bench-awk: $(TEST_FILE)
	hyperfine -m 500 --style basic --warmup 10 "gawk -f tests/utils/pattern.awk $(TEST_FILE) | ghead -n $(MAX_ITEMS)"

bench-bin: $(TEST_FILE)
	cargo build --release && \
	hyperfine -m 1000 --style basic --warmup 10 "target/release/count --max-items $(MAX_ITEMS) $<"
