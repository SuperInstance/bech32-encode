# Bech32 Encode

**A Rust library for Bech32 and Bech32m encoding** — the human-readable address format used by Bitcoin SegWit (BIP-173) and Taproot (BIP-350).

## Why It Matters

Bech32 was designed specifically for cryptocurrency addresses to replace Base58Check. It offers:

- **Error detection**: BCH checksum guarantees detection of up to 4 errors (and ~99.9996% of 5+ error cases). Base58Check only catches transcription errors via a single SHA-256 checksum.
- **Case-insensitive**: Unlike Base58, Bech32 is case-insensitive (uppercase = lowercase), making it easier to read and type.
- **No ambiguous characters**: Uses a 32-character alphabet (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`) with no 0/O/I/1/l confusion.
- **Separator character**: The `1` after the human-readable part (e.g., `bc1...`) visually separates the prefix from the data.

Bech32m is the improved variant for Taproot (witness v1+) addresses — it uses a different checksum constant (`0x2BC830A3` instead of `1`) to fix a subtle length-extension weakness in the original Bech32.

## How It Works

**Polymod checksum**: Bech32 uses a BCH polynomial checksum over GF(2^5). The HRP (Human-Readable Part, like "bc" or "tb") is expanded by splitting each character into high 3 bits and low 5 bits, interleaved with a 0 separator. The data payload (5-bit groups) and 6 zero bytes are appended, and a polynomial evaluation produces the 6-character checksum.

**Bit conversion**: Raw bytes (8-bit) are converted to 5-bit groups for encoding. `convertbits()` handles the 8→5 conversion with optional padding, and 5→8 for decoding without padding.

**SegWit address encoding**: A witness program is encoded as `[witver] || convertbits(witprog, 8, 5)`, then Bech32 (v0) or Bech32m (v1+) encoded with the HRP ("bc" for mainnet, "tb" for testnet).

## Quick Start

```rust
use bech32_encode::{encode_segwit_address, Encoding};

// P2WPKH address (witness v0, 20-byte program) — uses Bech32
let pubkey_hash: Vec<u8> = vec![0x75; 20];
let address = encode_segwit_address("bc", 0, &pubkey_hash);
println!("P2WPKH: {}", address);

// P2TR Taproot address (witness v1, 32-byte program) — uses Bech32m
let taproot_key: Vec<u8> = vec![0xAB; 32];
let taproot_addr = encode_segwit_address("bc", 1, &taproot_key);
println!("P2TR: {}", taproot_addr);

// Testnet
let testnet_addr = encode_segwit_address("tb", 0, &pubkey_hash);
println!("Testnet: {}", testnet_addr);
```

## API

- **`bech32_encode(hrp, data, encoding)` → `String`** — Encode with Bech32 or Bech32m
- **`bech32_verify_checksum(hrp, data)` → `Option<Encoding>`** — Verify and detect variant
- **`encode_segwit_address(hrp, witver, witprog)` → `String`** — Full SegWit address encoding
- **`convertbits(data, frombits, tobits, pad)` → `Option<Vec<u8>>`** — 8↔5 bit group conversion
- **`Encoding`** — Enum: `Bech32` (BIP-173), `Bech32m` (BIP-350)

## Architecture Notes

Provides cryptocurrency address encoding for SuperInstance blockchain tooling. Implements both BIP-173 (Bech32) and BIP-350 (Bech32m) with automatic variant selection by witness version. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
