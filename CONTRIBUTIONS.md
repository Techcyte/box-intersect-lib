# Contribution Guidelines

We welcome:

- Bug reports
- Pull requests for bug fixes
- Documentation improvements
- New box algorithms/data structures
- New built binaries for differnet archtectures
- New language bindings
- Performance optimizations of existing algorithms (those that make reasonable speed/memory/complexity tradeoffs, i.e. an archtecture-specific optimization will only be accepted if the speed improvmenet is significant)

Please consult with us before designing more extensive modifications.

## Contributing to the codebase

### Coding

Contributing code is done through standard github methods:

1. Fork this repo
2. Commit your code
3. Submit a pull request. It will be reviewed by maintainers and they'll give feedback or make requests as applicable

### Considerations
- Make sure existing tests pass with `cd box-intersect-lib/ && cargo test --release`
- Make sure your new code is reasonably tested (not needed for documentation changes)
- For performance improvments and new algorithms, benchmarks should be updated
