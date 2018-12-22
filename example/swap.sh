#!/bin/bash

if [! -f src/main.rs.bak]; then
cargo expand > src/expanded.rs
mv src/main.rs src/main.rs.bak
mv src/expanded.rs src/main.rs
else
  mv src/main.rs src/expanded.rs
  mv src/main.rs.bak src/main.rs
fi
