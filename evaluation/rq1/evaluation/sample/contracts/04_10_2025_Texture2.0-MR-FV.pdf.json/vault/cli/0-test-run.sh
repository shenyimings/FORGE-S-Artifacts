#!/usr/bin/env bash

read POOL0 < ../../superlendy/local-test/tmp/pool0
read CURATOR_FAFATOR < ../../superlendy/local-test/tmp/curator_fafator

cargo run -- create-vault \
    --name "My Vault" \
    --description "It's a good vault" \
    --mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v \
    --distribution $POOL0:100 \
    --curator $CURATOR_FAFATOR

