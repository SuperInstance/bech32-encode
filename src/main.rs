use std::fmt;

/// Bech32 and Bech32m encoding/decoding per BIP-173 and BIP-350.
const BECH32_CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const BECH32_CONST: u32 = 1;
const BECH32M_CONST: u32 = 0x2BC830A3;

/// Polymod checksum for Bech32.
fn bech32_polymod(values: &[u8]) -> u32 {
    let mut chk: u32 = 1;
    for &v in values {
        let b = chk >> 25;
        chk = ((chk & 0x1FFFFFF) << 5) ^ v as u32;
        for i in 0..5 {
            if (b >> i) & 1 != 0 {
                chk ^= [(0x3B6A57B2), (0x26508E6D), (0x1EA119FA), (0x3D4233DD), (0x2A1462B3)][i];
            }
        }
    }
    chk
}

/// Expand HRP for checksum computation.
fn bech32_hrp_expand(hrp: &str) -> Vec<u8> {
    let mut ret = Vec::with_capacity(hrp.len() * 2 + 1);
    for b in hrp.bytes() {
        ret.push(b >> 5);
    }
    ret.push(0);
    for b in hrp.bytes() {
        ret.push(b & 31);
    }
    ret
}

/// Create checksum for given HRP and data, using the specified encoding constant.
fn bech32_create_checksum(hrp: &str, data: &[u8], const_val: u32) -> [u8; 6] {
    let mut values = bech32_hrp_expand(hrp);
    values.extend_from_slice(data);
    values.extend_from_slice(&[0u8; 6]);
    let polymod = bech32_polymod(&values) ^ const_val;
    let mut checksum = [0u8; 6];
    for i in 0..6 {
        checksum[i] = ((polymod >> (5 * (5 - i))) & 31) as u8;
    }
    checksum
}

/// Verify checksum for given HRP and data+checksum.
fn bech32_verify_checksum(hrp: &str, data: &[u8]) -> Option<Encoding> {
    let mut values = bech32_hrp_expand(hrp);
    values.extend_from_slice(data);
    let polymod = bech32_polymod(&values);
    if polymod == BECH32_CONST {
        Some(Encoding::Bech32)
    } else if polymod == BECH32M_CONST {
        Some(Encoding::Bech32m)
    } else {
        None
    }
}

/// Bech32 encoding variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Bech32,
    Bech32m,
}

/// Encode data as a Bech32 or Bech32m string.
pub fn bech32_encode(hrp: &str, data: &[u8], encoding: Encoding) -> String {
    let const_val = match encoding {
        Encoding::Bech32 => BECH32_CONST,
        Encoding::Bech32m => BECH32M_CONST,
    };
    let combined = bech32_create_checksum(hrp, data, const_val);
    let mut result = hrp.to_string();
    result.push('1');
    for &d in data {
        result.push(BECH32_CHARSET[d as usize] as char);
    }
    for &c in &combined {
        result.push(BECH32_CHARSET[c as usize] as char);
    }
    result
}

/// Convert 8-bit bytes to 5-bit groups.
pub fn convertbits(data: &[u8], frombits: u8, tobits: u8, pad: bool) -> Option<Vec<u8>> {
    let mut acc: u32 = 0;
    let mut bits: u8 = 0;
    let mut ret = Vec::new();
    let maxv: u8 = (1 << tobits) - 1;
    for &value in data {
        if value >> frombits != 0 {
            return None;
        }
        acc = (acc << frombits) | value as u32;
        bits += frombits;
        while bits >= tobits {
            bits -= tobits;
            ret.push(((acc >> bits) & maxv as u32) as u8);
        }
    }
    if pad {
        if bits > 0 {
            ret.push(((acc << (tobits - bits)) & maxv as u32) as u8);
        }
    } else if bits >= frombits || ((acc << (tobits - bits)) & maxv as u32) != 0 {
        return None;
    }
    Some(ret)
}

/// Encode a SegWit witness program into a bech32 address.
pub fn encode_segwit_address(hrp: &str, witver: u8, witprog: &[u8]) -> String {
    let encoding = if witver == 0 {
        Encoding::Bech32
    } else {
        Encoding::Bech32m
    };
    let mut data = vec![witver];
    data.extend(convertbits(witprog, 8, 5, true).unwrap());
    bech32_encode(hrp, &data, encoding)
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Encoding::Bech32 => write!(f, "Bech32"),
            Encoding::Bech32m => write!(f, "Bech32m"),
        }
    }
}

fn main() {
    // Encode a simple bech32 string
    let data: Vec<u8> = vec![0, 14, 20, 15, 7, 13, 26, 0, 25, 18, 6, 11, 13, 8, 21, 4, 20, 3, 17, 2, 29, 3, 12, 29, 3, 4, 15, 24, 20, 6, 14, 30, 22];
    let encoded = bech32_encode("bc", &data, Encoding::Bech32);
    println!("Bech32 encoded: {}...", &encoded[..20]);

    // Encode a P2WPKH address (witness v0, 20-byte program)
    let witprog: Vec<u8> = vec![0x75; 20];
    let addr = encode_segwit_address("bc", 0, &witprog);
    println!("P2WPKH address: {}", addr);

    // Encode a P2TR address (witness v1, 32-byte program)
    let taproot_prog: Vec<u8> = vec![0xAB; 32];
    let tap_addr = encode_segwit_address("bc", 1, &taproot_prog);
    println!("P2TR address: {}", tap_addr);

    // Test convertbits round-trip
    let original = vec![0xFF, 0x00, 0xAB, 0xCD];
    let five_bit = convertbits(&original, 8, 5, true).unwrap();
    let back = convertbits(&five_bit, 5, 8, false).unwrap();
    assert_eq!(&original[..], &back[..], "round-trip failed");
    println!("8→5→8 bit conversion round-trip OK");

    // Testnet example
    let test_addr = encode_segwit_address("tb", 0, &witprog);
    println!("Testnet P2WPKH: {}", test_addr);
}
