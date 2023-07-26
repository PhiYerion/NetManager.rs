use std::process::Command;

macro_rules! cmd {
    ($name:literal, $command:expr) => {{
        let output = std::process::Command::new("ip")
            .args(&$command)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    Ok(result.stdout)
                } else {
                    Err(String::from_utf8_lossy(&result.stderr).to_string())
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }};
}


pub fn up(interface: &str, address: &str, _gateway: &str) {
    

    cmd!("ip address add", ["address", "add", address, "brd", "+", "dev", interface]).unwrap();

    //if !cmd!("ip address add", ["address", "add", address, "brd", "+", "dev", interface]) { print!("error1") };
    //if !cmd!("ip route add", ["route", "add", "default", "via", gateway, "dev", interface]) { print!("error2") };

    let _output = Command::new("ip")
    .args(&["address", "flush", "dev", interface])
    .output();

}

pub fn down(interface: &str) {

    // ip address flush dev enp7s0
    let output: std::process::Output = Command::new("ip")
    .args(&["address", "flush", "dev", interface])
    .output()
    .expect("Failed to execute 'ip address flush' command");

    // Check if the command was executed successfully
    if !output.status.success() {
        eprintln!(
            "Error occurred while executing 'ip address flush': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // ip route flush dev enp7s0
    let output = Command::new("ip")
        .args(&["route", "flush", "dev", interface])
        .output()
        .expect("Failed to execute 'ip route flush' command");

    // Check if the command was executed successfully
    if !output.status.success() {
        eprintln!(
            "Error occurred while executing 'ip route flush': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // ip link set down enp7s0
    let output = Command::new("ip")
        .args(&["link", "set", "down", interface])
        .output()
        .expect("Failed to execute 'ip link set down' command");


    // Check if the command was executed successfully
    if !output.status.success() {
        eprintln!(
            "Error occurred while executing 'ip link set down': {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}