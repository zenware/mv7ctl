# mv7ctl

Userspace HID Driver software to control settings on USB microphones compatible
with Shure MV7 devices.

Use this project at your own risk, I don't know what I'm doing and I have spare
microphones. My approach to this project has been: "I can see USB communication
through Wireshark. Therefore I must be able to replicate it."

## Usage, Installation, Etc.

Currently this is all manually done. More notes in following PRs as I get more
of it working, and start making builds. Planning to experiment with the GitHub
"releases" functionality.

```
nix develop
cargo build
# copy to $HOME/bin
```
