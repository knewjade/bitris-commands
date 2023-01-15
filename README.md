# bitris-commands

A set of commands using https://github.com/knewjade/bitris.

This repository is still in the alpha version. Therefore, the interface is still unstable.

The current goal is to reimplement some commands from https://github.com/knewjade/solution-finder in Rust.
We plan to integrate some components created in the process.

Eventually, we plan to make documentation and samples available to users.


# Usage/Documents

Now you can refer to the `cargo doc` and [some examples](example/src).

The examples are like a startup guide and exhaustive.
Then, when more detail is needed, we recommend generating crate documentation.

Eventually, when crate documentation is available on the web, we plan to organize it.


# Current features

### Checks if PC is possible

A pattern can be used to process multiple piece orders at once.
The results can then be aggregated to get the PC success rate.
This feature was called `percent` in solution-finder.

- [Example](example/src/pc_possible.rs)
