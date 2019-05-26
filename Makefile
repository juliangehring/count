TMP_DIR=/Volumes/ramdisk
TEST_FILE=$(TMP_DIR)/top-1m-toplevel.txt
NUM_CORES=$(nproc --all)

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
benchmark: bench-unix bench-awk bench-bin

bench-unix: $(TEST_FILE)
	hyperfine -m 10 --style basic --warmup 1 "gsort --parallel=4 $< | uniq -c | gsort --parallel=4 -k1,1 -rn | head -n 10"

bench-awk: $(TEST_FILE)
	hyperfine -m 200 --style basic --warmup 3 "gawk -f tests/utils/pattern.awk $(TEST_FILE) | head -n 100"

bench-bin: $(TEST_FILE)
	cargo build --release && \
	hyperfine -m 200 --style basic --warmup 3 "target/release/count --top 100 $<"
