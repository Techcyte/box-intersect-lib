set -e .

(cd box-intersect-lib && cargo clippy)
(cd box-intersect-lib-py && cargo clippy)