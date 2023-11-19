## Restructure
Refactor this into five parts:
1. local configurations (rtnetlink) (library)
    This should only be comprised of singular functions. There should not be state or presumptions abuot input. For instance, it should not determine what mac address to set even though it should have a function that has the capability to find it.
2. handling network stuff (dhcp, arp, etc) (library)
    This shoudn't have state or presumptions either.
3. configuration state manager (library)
    This has functions for modifying configuration and making sure that changes are properly propoated though the network and local configuration and stay that way. This will handle persistence and will call functions from the public api of local configuration and network stuff.
4. public api (binary / program)
    This will be a consistent abstraction layer upon the configuration state manager, reducing the amount of major releases
5. CLI interface (binary / program)
    This will just provide a cli interface to interact with the configuration state manager

## Features
1. Mac address options
    a. random mac address (including proper changes to mimic real manufacturer addresses)
    b. random mac address from preconfigured lists for specific kinds of devices (router, desktop, laptop, iot, etc)
        I. https://standards-oui.ieee.org/
2. in-depth network logs
    a. Find all devices on the network
    b. Generate pcap files for specific devices
    c. Log of ips and mac addresses associated with them
    d. Router responses and changes
3. in-depth network configuration
    a. Request address (ask router for a specific address)
    b. Presume address (act like we already have the address)
    c. Respect or do not respect router configuration responses
4. Fast startup
    Presume an address, then ping the address along with other backup addresses. If the presumed address respondes, then pivot to backup address that didn't respond. After we have presumed an address that doesn't respond, then send a request to the router for that address
5. Fast recover
    a. Maintain a backup connection with a different mac and ip address. Make sure that the connection remains alive. If there is a fault in the connection, fail over.
    b. Maintain a backup connection to a different wifi network (and follow same steps as above)
    c. If there is a fault in the connection and other steps don't work, find if we can hear from the network at all. If we can, carry out a full dhcp transaction and configure safely.
6. High throughput
    Connect to multiple wifi networks and distribute connections over them.
7. High verbosity
    Inform the client of router responses and keep logs
8. Seemingly multiple devices on one interface

## Penetration testing
1. Mimic the router and modify dhcp response to incorrect responses, like a malicious DNS server. This is inherantly a race condition and as such this requires a fast response to beat the router
2. ARP spoofing
