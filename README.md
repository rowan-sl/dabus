# DABus

## TODO's

- [x] figure out what the heck this standard is going to be
- [ ] figure out the standard two electric bogaloo
- [x] deal with multiple handlers handling the same message type (in a better way than panicking)
- [x] implement deregistering event handlers ~~once [trait upcasting](https://github.com/rust-lang/rust/issues/65991) is implemented~~
- [x] make BusEvent not store an option, and be consumed by the `is_into` and `into_raw` methods
- [x] **IMPORTANT**: make handler locating rely on the handlers type, not just if the type matches, so that some mismatch errors can never occur (possibly change so that it just checks return type instead?)
- [x] figure out how the previous TODO would interact with multiple handlers accepting the same event?
- [x] documentation *sigh*
- [ ] backtraces, so debugging of nested calls is acutally reasonable *double sigh*
- [ ] tests *triple sigh*
- [ ] figure out if there should be a distinction between send and query events anymore, and remove it if there is not
- [ ] document the differences between Send and Query events

## TODO's v2wos

- [ ] docs
- [ ] tests
- [ ] backtraces (do LATER, do logging NOW)
- [x] proper error handling
- [ ] multi-handler events
- [ ] more complex event matching (allow handlers to consume an event, after looking at the arguments?)
- [x] nested handler calls
- [ ] error forwarding
