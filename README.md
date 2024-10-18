## Arkworks Integration

This repo is meant to demonstrate compatibility with the arkworks/circom toolchain [circom-compat](https://github.com/arkworks-rs/circom-compat).
It contains a single executable which generates a witness for the [factor circuit](https://github.com/l-adic/factors?tab=readme-ov-file#factors) that appears in other ℓ-adic demo projects.

# How to run

Simply run
```
> cargo run
```
and you will see the witness printed to standard out. You can change the `inputs.json` file the change the public inputs.

We omit demonstrating the proof generation / ethereum integrations in this example because it follows from the compatibility with circom-compat.
