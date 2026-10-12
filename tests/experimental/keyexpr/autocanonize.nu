#!/usr/bin/env nuze -X0

use std/assert

# NOTE: see https://github.com/eclipse-zenoh/roadmap/blob/main/rfcs/ALL/Key%20Expressions.md#canon-forms

assert equal (z keyexpr autocanonize "hello/foo$*$*/bar") "hello/foo$*/bar"
assert equal (z keyexpr autocanonize "hello/**/**/bye") "hello/**/bye"
assert equal (z keyexpr autocanonize "hello/$*$*/bye") "hello/*/bye"
assert equal (z keyexpr autocanonize "hello/**/*") "hello/*/**"
