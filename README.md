# NetManager.rs

This is a network manager for Rust. It is supposed to fill a similar role to NetworkManager on Linux, but it has a different paradigm. Instead of either merely doing the bidding of DHCP or manually configuring settings, it takes both into account and provides a spectrum of control. For instance, it will get the subnet range from DHCP and let the user decide what IP they want from that range (currently implemented).


## Planned features
Find conflicting IP addresses and notify the user of those conflicts.
Test configurations in a container with the host network to make sure that it works

Integrate https://maclookup.app/downloads/json-database into mac setter

## Build
`git clone https://github.com/PhiYerion/NetManager.rs; cd NetManager.rs; chmod +x ./build.sh; ./build.sh`
or `git clone https://github.com/PhiYerion/NetManager.rs; cd NetManager.rs; cargo build`

## Usage
This is a library that happens to have a main for testing. This is very suboptimal and is planned to be abstracted into three different modules.
