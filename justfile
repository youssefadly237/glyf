gen:
    python3 scripts/build_corpus.py

gen-unicode:
    python3 scripts/build_corpus.py --no-nerd-fonts

check:
    cargo clippy

bench:
    cargo bench

bench-cli:
    ./benches/cli.sh

clean:
    cargo clean
    rm -f data/corpus.tsv data/blocks.tsv
    rm -rf data/raw
