#!/bin/bash
programId=$(solana address -k target/deploy/sallar-keypair.json) && \
    sed -i 's/sallar = ".*/sallar = "'"$programId"'"/' Anchor.toml && \
    sed -i 's/declare_id!.*/declare_id!("'"$programId"'");/' programs/sallar/src/lib.rs && \
    anchor build

while pkill -9 solana-test-val; do
    sleep 1
done

for TEST_SUITE in "$@"
do
    echo "Executing test suite: $TEST_SUITE"
    solana-test-validator -r --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s programs/sallar/tests/fixtures/mpl_token_metadata.so -q &
    sleep 5
    anchor deploy

    anchor run $TEST_SUITE

    while pkill -9 solana-test-val; do
        sleep 1
    done
    rm -rf test-ledger
done
