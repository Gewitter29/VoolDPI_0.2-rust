# VoolDPI

A real-time TCP packet fragmentation and injection engine written in Rust for DPI analysis and bypass.

## Overview

**VoolDPI** is a low-level network utility designed to capture, analyze, and manipulate TCP/IP packets on the fly. It specifically targets ClientHello packets to perform real-time segment splitting and packet injection, serving as a practical experiment in network stack manipulation and traffic optimization using Rust.

## Features

- **Real-time Packet Capture:** Uses `pnet` to interface directly with the network data link layer.
- **Protocol Inspection:** Filters and inspects IPv4 and TCP layers to identify specific TLS handshake patterns (ClientHello).
- **TCP Fragmentation / Injection:** Splits packet payloads dynamically, recalculates IPv4/TCP checksums, and injects modified segments back into the network stream.
- **Memory Safe & Performant:** Built entirely in Rust for zero-cost abstractions and low-level memory safety.

## Tech Stack

- **Language:** Rust
- **Crates:** `pnet` (Network packet manipulation and datalink channels)

## Usage & License

1. Clone the repository and navigate into the directory:
   ```bash
   git clone [https://github.com/Gewitter29/VoolDPI_0.2-rust.git](https://github.com/Gewitter29/VoolDPI_0.2-rust.git)
   cd VoolDPI_0.2-rust

 2. Build and run the project (requires administrative/root privileges for raw socket access):
    Bash

    cargo build --release
    sudo ./target/release/vooldpien_rs

    License: This project is open-source and available under the MIT License.
