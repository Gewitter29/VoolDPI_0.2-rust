use pnet::datalink::{self, Channel};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{Ipv4Packet, MutableIpv4Packet, checksum as ipv4_checksum};
use pnet::packet::tcp::{TcpPacket, MutableTcpPacket, ipv4_checksum as tcp_ipv4_checksum};
use pnet::packet::{Packet, MutablePacket};

fn find_active_interface(interfaces: &[datalink::NetworkInterface]) -> Option<datalink::NetworkInterface> {
    interfaces.iter().find(|iface| {
        iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty()
    }).cloned()
}

fn main() {
    println!("[-] VoolDPI (Real-time Fragmentation and Injection Engine) starting...");

    let interfaces = datalink::interfaces();
    let interface = match find_active_interface(&interfaces) {
        Some(iface) => iface,
        None => {
            println!("[-] Active and suitable network interface not found!");
            return;
        }
    };

    println!("[+] Listening network interface: {} ({})", interface.name, interface.mac.unwrap_or_default());

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            println!("[-] Unknown channel type!");
            return;
        }
        Err(e) => {
            println!("[-] Network channel could not be opened: {:?}", e);
            return;
        }
    };

    println!("[+] DPI Bypass and segment splitting engine active...\n");

    loop {
        match rx.next() {
            Ok(packet) => {
                let ethernet_packet = EthernetPacket::new(packet);
                if let Some(eth) = ethernet_packet {
                    if eth.get_ethertype() == EtherTypes::Ipv4 {
                        if let Some(ipv4_packet) = Ipv4Packet::new(eth.payload()) {
                            if ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
                                if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                    let tcp_payload = tcp_packet.payload();

                                    if tcp_payload.len() > 5 && tcp_payload[0] == 0x16 && tcp_payload[5] == 0x01 {
                                        if ipv4_packet.get_source().is_private() {
                                            println!(
                                                "[+] ClientHello Caught | IP: {} -> {} | Port: {} -> {}",
                                                ipv4_packet.get_source(),
                                                ipv4_packet.get_destination(),
                                                tcp_packet.get_source(),
                                                tcp_packet.get_destination()
                                            );
                                        }

                                        if tcp_payload.len() > 18 {
                                            let split_index = 3;
                                            let part1 = &tcp_payload[..split_index];
                                            let part2 = &tcp_payload[split_index..];

                                            let eth_len = 14;
                                            let ip_header_len = (ipv4_packet.get_header_length() * 4) as usize;
                                            let tcp_header_len = (tcp_packet.get_data_offset() * 4) as usize;

                                            let mut pkt1_data = vec![0u8; eth_len + ip_header_len + tcp_header_len + part1.len()];
                                            pkt1_data[..eth_len].copy_from_slice(&packet[..eth_len]);

                                            let mut new_ip1 = MutableIpv4Packet::new(&mut pkt1_data[eth_len..]).unwrap();
                                            new_ip1.clone_from(&ipv4_packet);
                                            let total_len1 = (ip_header_len + tcp_header_len + part1.len()) as u16;
                                            new_ip1.set_total_length(total_len1);
                                            new_ip1.set_checksum(ipv4_checksum(&new_ip1.to_immutable()));

                                            let mut new_tcp1 = MutableTcpPacket::new(&mut pkt1_data[eth_len + ip_header_len..]).unwrap();
                                            new_tcp1.clone_from(&tcp_packet);
                                            new_tcp1.set_payload(part1);
                                            new_tcp1.set_checksum(tcp_ipv4_checksum(
                                                &new_tcp1.to_immutable(),
                                                &ipv4_packet.get_source(),
                                                &ipv4_packet.get_destination(),
                                            ));

                                            let mut pkt2_data = vec![0u8; eth_len + ip_header_len + tcp_header_len + part2.len()];
                                            pkt2_data[..eth_len].copy_from_slice(&packet[..eth_len]);

                                            let mut new_ip2 = MutableIpv4Packet::new(&mut pkt2_data[eth_len..]).unwrap();
                                            new_ip2.clone_from(&ipv4_packet);
                                            let total_len2 = (ip_header_len + tcp_header_len + part2.len()) as u16;
                                            new_ip2.set_total_length(total_len2);
                                            new_ip2.set_checksum(ipv4_checksum(&new_ip2.to_immutable()));

                                            let mut new_tcp2 = MutableTcpPacket::new(&mut pkt2_data[eth_len + ip_header_len..]).unwrap();
                                            new_tcp2.clone_from(&tcp_packet);
                                            new_tcp2.set_sequence(tcp_packet.get_sequence() + part1.len() as u32);
                                            new_tcp2.set_payload(part2);
                                            new_tcp2.set_checksum(tcp_ipv4_checksum(
                                                &new_tcp2.to_immutable(),
                                                &ipv4_packet.get_source(),
                                                &ipv4_packet.get_destination(),
                                            ));

                                            if let Some(Ok(_)) = tx.send_to(&pkt1_data, None) {
                                                if let Some(Ok(_)) = tx.send_to(&pkt2_data, None) {
                                                    println!("[-] Packet Split and Injected: {} -> {}", part1.len(), part2.len());
                                                }
                               3             }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("[-] Packet read error: {:?}", e);
            }
        }
    }
}
