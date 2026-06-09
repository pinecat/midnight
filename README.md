# midnight

Schedule emails to be sent at a later time via
[at(1)](https://linux.die.net/man/1/at).

## Installation

### From [crates.io](https://crates.io/crates/midnight)

```sh
cargo install midnight
```

### From source

```sh
git clone https://github.com/pinecat/midnight
cd midnight
cargo install --path .
```

### Runtime dependencies

You will also need `at` on your system. If you do not have it, grab it
from your package manager.

If you are on macOS, the `at` daemon is not enabled by default, so you
will need to run the following command to enable and start it:

```
sudo launchctl enable system/com.apple.atrun
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

### Draftboxes

You may specify which draftbox an account is associated with, using a
.draftboxes file. If you do not specify a .draftboxes files, `midnight`
will grep around your neomutt config directory, and attempt to find the
correct draftbox using the `postponed = ...` option inside a neomutt
config file. If you use multiple accounts, you must have a separate file
for each account for this to work properly. An example .draftboxes is
below. Lines starting with '#' are comments.

```
# ~/.config/neomutt/.draftboxes
noreply@example.com = ~/mail/example.com/drafts
notarealaddress@crates.io = ~/mail/crates.io/draftbox
```

## Usage

### Programs

Installing midnight will give you access to the following programs:

- `midnight`: Used to add a message to the queue. Primarily intended to
be used from neomutt via a macro. See the `Configuration` section
above for details.
- `mn`: Alias for `midnight`.
- `mnq`: List mail that's been scheduled for delivery.
- `mnrm`: Remove a scheduled mail from the queue by passing it's job ID
(this ID appears in `mnq` as the first value, between square
brackets).
- `mnsend`: Send a message using a unique message ID. This program is
not intended to be called by the user, but rather, is used by `midnight`
internally when creating a new job in at(1).

### Getting help

You can run `midnight -h` to get a help menu that displays proper
program usage, as well as some optional flags. The flags may be used
with any of the binaries listed above, and may override that binary's
default behavior.

### Usage in neomutt

On the compose screen, use `P` to postpone the message. Then, go to your
drafts folder, and use the macro you set above to schedule the message
to be sent later (or see the `Advanced configuration` section on
possibly setting a macro to send later directly from the compose menu).
You will be prompted to enter a time at which to schedule the message
for delivery. You may enter any time that can be interpreted by at(1).
For instance:

- now + 10 minutes
- 0600
- 2:00pm
- 1800 Jun 1 2030

See `man at` for more details. If the time you enter is unable to be
interpreted properly, the program will quit with an error message, and
your email will not be added to the queue.

## Caveats

This software is in a beta release, and there are some quircks and
missing features for the time being:

- Minimal configuration options (a .draftboxes file). If you want to
change how the program invokes at, for instance, you will have to change
the source code for now.
- ~~If you are using multiple accounts in neomutt, midnight will assume
that each account has it's own file. This is important for reading the
correct `folder` and `postponed` values inside of your config. If you
don't have separate files for each account, then for now, you will need
to either refactor your config, or change the source code of the program
so it works with your setup.~~ You may now set a .draftboxes file (see
Draftboxes section above for details). By default, `midnight` will first
try to look for this file, then fallback to grepping your neomutt config
if this file cannot be found.
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
git clone https://github.com/pinecat/midnight
cd midnight
cargo build
```
