use ethers::abi::Address;

use std::collections::HashMap;

#[derive(Debug)]
pub enum Flag {
    CTFSimple(bool),
    MagicNumberV1(bool),
    MagicNumberV2(bool),
    MagicNumberV3(bool),
    NewContracts(bool),
    CTFTriple(bool),
} //bool is whether I have completed -> bit janky

pub fn contracts() -> HashMap<Address, Flag> {
    HashMap::from([
        (
            "0x65459dd36b03af9635c06bad1930db660b968278"
                .parse::<Address>()
                .unwrap(),
            Flag::CTFSimple(true),
        ),
        (
            "0x98997b55bb271e254bec8b85763480719dab0e53"
                .parse::<Address>()
                .unwrap(),
            Flag::CTFSimple(true),
        ),
        (
            "0x1cddb0ba9265bb3098982238637c2872b7d12474"
                .parse::<Address>()
                .unwrap(),
            Flag::CTFSimple(true),
        ),
        (
            "0x118bcb654d9a7006437895b51b5cd4946bf6cdc2"
                .parse::<Address>()
                .unwrap(),
            Flag::MagicNumberV1(true),
        ),
        (
            "0x9be957d1c1c1f86ba9a2e1215e9d9eefde615a56"
                .parse::<Address>()
                .unwrap(),
            Flag::MagicNumberV2(true),
        ),
        (
            "0xe8b7475e2790409715af793f799f3cc80de6f071"
                .parse::<Address>()
                .unwrap(),
            Flag::MagicNumberV3(true),
        ),
        (
            "0x5eA0feA0164E5AA58f407dEBb344876b5ee10DEA"
                .parse::<Address>()
                .unwrap(),
            Flag::NewContracts(true),
        ),
        (
            "0x1ea6fb65bab1f405f8bdb26d163e6984b9108478"
                .parse::<Address>()
                .unwrap(),
            Flag::CTFTriple(true),
        ),
        (
            "0x20a1A5857fDff817aa1BD8097027a841D4969AA5"
                .parse::<Address>()
                .unwrap(),
            Flag::CTFSimple(true),
        ),
    ])
}
