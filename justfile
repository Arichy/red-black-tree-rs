test-debug:
    # cargo nextest r --test integration_test test_bst_property_maintained_during_deletions
    RUST_BACKTRACE=1 cargo nextest r --test prop_test minimal --no-capture

integration_test:
    RUST_BACKTRACE=1 cargo nextest r --test integration_test

prop_test:
    PROPTEST_CASES=10 cargo test --test prop_test -- --nocapture

test:
    PROPTEST_CASES=13 cargo nextest r
