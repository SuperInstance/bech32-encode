# bech32-encode

**Bech32 and Bech32m encoding per BIP-173 and BIP-350 — SegWit address generation with BCH error-detecting codes.**

Bech32 is a human-readable binary-to-text encoding designed specifically for Bitcoin SegWit addresses. It uses a 32-character alphabet (all lowercase, no ambiguous symbols) and a **BCH error-detecting code** that catches up to 4 character substitutions or transpositions with probability > 99.9999%. Bech32m (BIP-350) fixes a subtle length-extension weakness in the original Bech32 checksum for witness version 1+ (Taproot) addresses.

## Why It Matters

Bech32 replaced Base58Check for SegWit addresses because:

- **No mixed case** — Bech32 is lowercase-only, eliminating copy/paste errors from case sensitivity.
- **No ambiguous characters** — Uses `qpzry9x8gf2tvdw0s3jn54khce6mua7l`, excluding `1`, `b`, `i`, `o` (easily confused with `l`, `6`, etc.).
- **Error detection** — The 30-bit checksum detects any ≤4 character error and most longer bursts. Error correction is possible but intentionally not used (correction can mask typos that should be rejected).
- **QR code efficiency** — Lowercase alphanumeric QR encoding is more compact than mixed-case Base58.

Bech32 is used for:

- **P2WPKH addresses** — `bc1q...` (witness v0, Bech32)
- **P2WSH addresses** — `bc1q...` (witness v0, Bech32)
- **P2TR addresses** — `bc1p...` (witness v1, Bech32m)
- **Lightning Network invoices** — BOLT-11 uses Bech32

## How It Works

### Structure

A Bech32 string has the format:

```
hrp + '1' + data_part + checksum
```

- **hrp** (Human-Readable Part): e.g., `bc` (mainnet), `tb` (testnet) — 1–83 characters, US-ASCII 33–126.
- **Separator**: `1` (not in the data alphabet, so it's unambiguous).
- **Data part**: 5-bit groups mapped to the Bech32 charset.
- **Checksum**: 6 characters (30 bits).

### Checksum Algorithm

The checksum is a **BCH code** over GF(2^5) with a custom generator polynomial. The polymod function:

```
GEN = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3]

polymod(values):
    chk = 1
    for v in values:
        b = chk >> 25
        chk = ((chk & 0x1ffffff) << 5) ^ v
        for i in 0..5:
            if (b >> i) & 1: chk ^= GEN[i]
    return chk
```

The checksum constants ensure:

| Encoding | Constant | Check equation |
|----------|----------|----------------|
| Bech32 | 1 | polymod(hrp_expand ++ data ++ checksum) = 1 |
| Bech32m | 0x2bc830a3 | polymod(hrp_expand ++ data ++ checksum) = 0x2bc830a3 |

The HRP is expanded by interleaving high bits and low bits:

> hrp_expand = [ord(c) >> 5 for c in hrp] ++ [0] ++ [ord(c) & 31 for c in hrp]

This ensures the checksum is sensitive to the HRP (a `bc` address can't be confused with a `tb` address).

### 8-to-5 Bit Conversion

Data bytes (8-bit) are packed into 5-bit groups for the Bech32 alphabet:

```
acc = 0; bits = 0
for byte in data:
    acc = (acc << 8) | byte
    bits += 8
    while bits >= 5:
        bits -= 5
        emit((acc >> bits) & 0x1f)
if pad and bits > 0:
    emit((acc << (5 - bits)) & 0x1f)
```

The padding bit must be zero on decode (otherwise the conversion is invalid), ensuring canonical encoding.

### SegWit Address Format

```
address = hrp + '1' + [witver] + convertbits(witprog, 8, 5, pad=true) + checksum
```

- **witver** = witness version (0–16), stored as a single 5-bit value
- **witprog** = witness program (20 bytes for P2WPKH, 32 bytes for P2TR)

Encoding rule:
- witver 0 → Bech32 (BIP-173)
- witver 1+ → Bech32m (BIP-350)

### Error Detection Properties

The 30-bit checksum provides:

| Error type | Detectable? | Notes |
|-----------|-------------|-------|
| 1 char substitution | ✓ Always | |
| 2 char substitution | ✓ Always | |
| 3 char substitution | ✓ Always | |
| 4 char substitution | ✓ Always | |
| ≥5 char substitution | ~99.9999% | |
| 1 char transposition | ✓ Always | |
| 2 adjacent transposition | ✓ Always | |
| Length extension | ✓ (Bech32m) | Fixed by BIP-350 |

### Complexity

| Operation | Time | Notes |
|-----------|------|-------|
| `bech32_polymod(n values)` | O(n) | Per-value constant work |
| `bech32_encode(hrp, data)` | O(n) | Encode + checksum |
| `convertbits(n bytes)` | O(n) | Bit packing |
| `encode_segwit_address` | O(n) | Full address generation |

## Quick Start

```rust
// This crate runs as a binary demo
// Run: cargo run

use bech32_encode::{bech32_encode, encode_segwit_address, Encoding, convertbits};

// P2WPKH address (witness v0 = Bech32)
let witprog = vec![0x75u8; 20];
let addr = encode_segwit_address("bc", 0, &witprog);
println!("P2WPKH: {}", addr);  // bc1q...

// P2TR address (witness v1 = Bech32m)
let taproot_prog = vec![0xABu8; 32];
let tap_addr = encode_segwit_address("bc", 1, &taproot_prog);
println!("P2TR: {}", tap_addr);  // bc1p...

// Testnet
let test_addr = encode_segwit_address("tb", 0, &witprog);

// 8-to-5 bit round-trip
let original = vec![0xFF, 0x00, 0xAB, 0xCD];
let five_bit = convertbits(&original, 8, 5, true).unwrap();
let back = convertbits(&five_bit, 5, 8, false).unwrap();
assert_eq!(&original[..], &back[..]);
```

## API

- **`bech32_encode(hrp, data, encoding) → String`** — Full Bech32/Bech32m encoding
- **`encode_segwit_address(hrp, witver, witprog) → String`** — SegWit address with auto-variant selection
- **`convertbits(data, frombits, tobits, pad) → Option<Vec<u8>>`** — Base conversion between bit widths
- **`bech32_polymod(values) → u32`** — BCH checksum polynomial evaluation
- **`bech32_create_checksum(hrp, data, const_val) → [u8; 6]`** — 6-character checksum
- **`bech32_verify_checksum(hrp, data) → Option<Encoding>`** — Verify and detect variant
- **`Encoding`** — Bech32, Bech32m

## Architecture Notes

The γ+η=C identity: γ (generative capacity) is the range of addresses the format can express — any witness version (0–16) with any program length (2–40 bytes). η (evaluative depth) is the checksum's error-detecting power — 30 bits catching all ≤4 errors. C = addressing reliability: a system that can express any SegWit output type (high γ) while catching virtually all transcription errors (high η) achieves near-perfect C (trustworthy address communication).

## References

1. Wuille, P. (2017). BIP-173: "Base32 address format for native v0-16 witness outputs."
2. Wuille, P. (2020). BIP-350: "Validation of Bech32 and Bech32m addresses." — Fixes length-extension bug.
3. Bossert, H. (2017). "Analysis of Bech32 checksum properties." — BCH code design rationale.
4. Lin, S. & Costello, D. (2004). *Error Control Coding* (2nd ed.). — BCH code theory underlying the checksum.

## License

MIT
