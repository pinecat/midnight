# midnight

Schedule emails to be sent at a later time via
[at(1)](https://linux.die.net/man/1/at). For now, also depends on
[ripgrep](https://github.com/BurntSushi/ripgrep).

## Installation

The package is available from [crates.io](https://crates.io/crates/midnight):

```sh
cargo install midnight
```

Then in your neomutt config, assign a macro to call the binary. Make
sure it's in your path somewhere.

```
macro index,pager L "|midnight<enter>" "send later"
```

## Usage

On the compose screen, use `P` to postpone the message. Then, go to your
drafts folder, and use the macro your set above to schedule the message
to be sent later.

## Caveats

This software is in a beta release, and there are some quircks and
missing features for the time being:

- No configuration options. If you want to change how the program
invokes at, or ripgrep, for instance, you will have to change the source
code for now.
- There is no state management, so there is no way to tell what job in
atq correponds to what message in drafts.
- There is (effectively) no way to call the program from the compose
menu in neomutt at the moment (which would be very useful).
- The program does not delete the message in drafts after the message is
sent (however, the sent message will be displayed in your sent folder).

## Building

```sh
git clone git@library.cat:rory/midnight
cd midnight
cargo build
```
