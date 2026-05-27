use sysinfo::Networks;

#[test]
fn test_networks() {
    let networks = Networks::new_with_refreshed_list();
    for (_name, network) in &networks {
        if network.mac_address() == sysinfo::MacAddr::UNSPECIFIED {
            println!("Unspecified");
        }
    }
}
