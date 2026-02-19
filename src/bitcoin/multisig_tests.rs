use crate::bitcoin::multisig::create_2of3_multisig;
use bitcoin::{Network, PublicKey};

#[test]
fn creates_valid_2of3_multisig() {
    let k1: PublicKey = "02c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7"
        .parse().unwrap();
    let k2: PublicKey = "03c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7"
        .parse().unwrap();
    let k3: PublicKey = "02abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef"
        .parse().unwrap();

    let (address, script) =
        create_2of3_multisig(vec![k1, k2, k3], Network::Regtest);

    assert!(address.to_string().starts_with("bcrt1"));
    assert!(script.to_bytes().ends_with(&[0xae])); // OP_CHECKMULTISIG
}

#[test]
fn creates_deterministic_multisig() {
    let k1: PublicKey = "02c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7"
        .parse().unwrap();
    let k2: PublicKey = "03c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7"
        .parse().unwrap();
    let k3: PublicKey = "02abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef"
        .parse().unwrap();

    let (addr1, script1) = create_2of3_multisig(vec![k1, k2, k3], Network::Regtest);
    let (addr2, script2) = create_2of3_multisig(vec![k3, k1, k2], Network::Regtest);

    assert_eq!(addr1, addr2);
    assert_eq!(script1, script2);
}
