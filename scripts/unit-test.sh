set -e .

(cd box-intersect-lib && cargo test)
(cd box-intersect-lib-py && cargo test)