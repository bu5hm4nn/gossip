#!/bin/bash

RUSTFLAGS="-C target-cpu=native --cfg tokio_unstable" cargo build --features=lang-cjk,video-ffmpeg,side-menu --release && \
    RUST_BACKTRACE=1 RUST_LOG="info,gossip=debug" ./target/release/gossip

