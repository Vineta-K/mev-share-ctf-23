use ethers_contract::Abigen;

fn main() {
    Abigen::new("MevShareCTFSimple", "./abi/mev_share_ctf_simple.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("./src/abi/mev_share_ctf_simple.rs")
        .unwrap();

    Abigen::new("MevShareCTFTriple", "./abi/mev_share_ctf_triple.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("./src/abi/mev_share_ctf_triple.rs")
        .unwrap();

    Abigen::new(
        "MevShareMagicNumber",
        "./abi/mev_share_magic_number_v3.json",
    )
    .unwrap()
    .generate()
    .unwrap()
    .write_to_file("./src/abi/mev_share_magic_number_v3.rs")
    .unwrap();

    Abigen::new("MevShareNewContracts", "./abi/mev_share_new_contracts.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("./src/abi/mev_share_new_contracts.rs")
        .unwrap();

    Abigen::new("MevShareNewContract", "./abi/mev_share_new_contract.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("./src/abi/mev_share_new_contract.rs")
        .unwrap();
}
