# DABus

## TODO's

- [x] figure out what the heck this standard is going to be
- [ ] figure out the standard two electric bogaloo
- [x] deal with multiple handlers handling the same message type (in a better way than panicking)
- [ ] implement deregistering event handlers once [trait upcasting](https://github.com/rust-lang/rust/issues/65991) is implemented
- [x] make BusEvent not store an option, and be consumed by the `is_into` and `into_raw` methods
- [ ] **IMPORTANT**: make handler locating rely on the handlers type, not just if the type matches, so that some mismatch errors can never occur (possibly change so that it just checks return type instead?)
- [ ] figure out how the previous TODO would interact with multiple handlers accepting the same event?
- [ ] documentation *sigh*
- [ ] backtraces, so debugging of nested calls is acutally reasonable *double sigh*
- [ ] tests *triple sigh*
- [ ] figure out if there should be a distinction between send and query events anymore, and remove it if there is not
