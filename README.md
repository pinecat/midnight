# midnight

Schedule emails to be sent at a later time via
[at(1)](https://linux.die.net/man/1/at). For now, also depends on
[ripgrep](https://github.com/BurntSushi/ripgrep).

## Installation

### From [crates.io](https://crates.io/crates/midnight)

```sh
cargo install midnight
```

### From source

```sh
git clone git@library.cat:rory/midnight
cd midnight
cargo install --path .
```

### Configuration

In your neomutt config, assign a macro to call the binary. Make sure
it's in your path somewhere.

```
macro index,pager L "|midnight<enter>" "send later"
```

You must also ensure that you are using `pipe_decode = no` somewhere in
your neomutt config, as `midnight` uses the message ID to send your
email (setting `pipe_decode` strips the message ID). This ensures your
scheduled (queued) messages get sent, even if you decide to edit their
contents before they are delivered.

### Advanced configuration

Depending on your setup, it may be possible to define a macro that works
from the compose menu. You may need to set a separate macro per account,
if your accounts use different names for the draft boxes. Something like
the command below will likely work:

```
macro compose L "<postpone-message><enter><change-folder>=Drafts<enter>|midnight<enter>"
```

This seems to function ok in my setup, as the lastest postponed message
is always be the first message my cursor lands on when switching to the
drafts box. However, this may not be the case for you, depending on some
other options in your neomutt config. So, please be careful, and test
this yourself before relying on it.

## Usage

On the compose screen, use `P` to postpone the message. Then, go to your
drafts folder, and use the macro your set above to schedule the message
to be sent later (or see the `Advanced configuration` section on
possibly setting a macro to send later directly from the compose menu).

## Caveats

This software is in a beta release, and there are some quircks and
missing features for the time being:

- No configuration options. If you want to change how the program
invokes at, or ripgrep, for instance, you will have to change the source
code for now.
- ~~There is no state management, so there is no way to tell what job in
atq correponds to what message in drafts.~~ We have state manage now!
`midnight` keeps some metadata in it's own queue file. By default, it's
stored in `NEOMUTT_XDG_CONFIG_DIR/.midnight`. Additionally, calling
`mnrm <jobid>` will also remove the job from `atq(1)`.
- ~~There is (effectively) no way to call the program from the compose
menu in neomutt at the moment (which would be very useful).~~ There is
currently a hack/workaround for this, though it may or may not work for
you. Please see the `Advanced configuration` section for more details.
- ~~The program does not delete the message in drafts after the message
is sent (however, the sent message will be displayed in your sent
folder).~~ Due to the extra metadata storage, and calling `mnsend`
instead of invoking `neomutt` in `at(1)` directly, we now (attempt) to
delete the message in the user's draftbox after it is sent.

## Building

```sh
git clone git@library.cat:rory/midnight
cd midnight
cargo build
```
