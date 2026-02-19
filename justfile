update-cargo-lock:
  cargo generate-lockfile
  cp Cargo.lock Cargo.nix.lock
