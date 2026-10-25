//! Operator tool for privacy-preserving registry attestations.
//!
//! Four subcommands, matching the four things an operator actually does:
//!
//! ```text
//! attest setup   --out DIR                        # once, per circuit version
//! attest root    --records FILE                   # publish this to governance
//! attest prove   --records FILE --index N
//!                --threshold K --pk DIR/pk.bin --out proof.bin
//! attest verify  --vk DIR/vk.bin --proof proof.bin
//!                --set-root HEX --threshold K --nullifier HEX
//! ```
//!
//! # Records file
//!
//! Tab-separated, one record per line, `#` starts a comment:
//!
//! ```text
//! # index  subject   value  tag  salt
//! 0        7         5000   1    99
//! 3        0x08      42     0x01 11
//! ```
//!
//! `subject`, `tag` and `salt` accept a decimal `u64`, a `0x`-prefixed hex
//! `u64`, or a full 64-character hex field element. `value` is a decimal `u64`
//! because it is compared numerically, and a 64-bit width is the limit the
//! circuit supports.
//!
//! # The operator never shares this file
//!
//! It is the witness. Anyone holding it can generate proofs for every record in
//! it. Publish only the root.
//!
//! # Do not use `setup` output in production
//!
//! See `setup_for_testing` in the library: Groth16 needs a multi-party ceremony,
//! and a locally generated CRS lets whoever ran this command forge proofs. The
//! command exists so the rest of the pipeline can be exercised end to end.

use std::collections::BTreeMap;
use std::process::ExitCode;

use belizechain_zk_attestation::{
    verify_attestation_inputs, AttestationCircuit, AttestationInputs, Fr, ProverError, Setup,
    SparseTree, VALUE_BITS,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        return usage();
    };
    let flags = match Flags::parse(&args[1..]) {
        Ok(flags) => flags,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };

    let result = match command {
        "setup" => cmd_setup(&flags),
        "root" => cmd_root(&flags),
        "prove" => cmd_prove(&flags),
        "verify" => cmd_verify(&flags),
        "--help" | "-h" | "help" => {
            usage();
            return ExitCode::SUCCESS;
        }
        other => {
            eprintln!("error: unknown command `{other}`");
            return usage();
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: attest <setup|root|prove|verify> [--flag value ...]\n\
         \n\
         setup   --out DIR\n\
         root    --records FILE\n\
         prove   --records FILE --index N --threshold K --pk FILE --out FILE\n\
         verify  --vk FILE --proof FILE --set-root HEX --threshold K --nullifier HEX"
    );
    ExitCode::FAILURE
}

/// Minimal `--flag value` parser. A dependency-free CLI is one less thing an
/// operator has to trust and one less thing to keep patched.
struct Flags {
    values: BTreeMap<String, String>,
}

impl Flags {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut values = BTreeMap::new();
        let mut iter = args.iter();
        while let Some(flag) = iter.next() {
            let Some(name) = flag.strip_prefix("--") else {
                return Err(format!("expected a --flag, found `{flag}`"));
            };
            let value = iter
                .next()
                .ok_or_else(|| format!("`--{name}` needs a value"))?;
            values.insert(name.to_string(), value.clone());
        }
        Ok(Self { values })
    }

    fn get(&self, name: &str) -> Result<&str, String> {
        self.values
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| format!("missing required `--{name}`"))
    }

    fn number(&self, name: &str) -> Result<u64, String> {
        self.get(name)?
            .parse()
            .map_err(|_| format!("`--{name}` must be a decimal number"))
    }
}

// --- records -------------------------------------------------------------

struct Record {
    index: u64,
    subject: Fr,
    value: u64,
    tag: Fr,
    salt: Fr,
}

fn load_records(path: &str) -> Result<Vec<Record>, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    let mut records = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        if fields.len() != 5 {
            return Err(format!(
                "{path}:{}: expected 5 tab-separated fields, found {}",
                line_number + 1,
                fields.len()
            ));
        }
        let parse_u64 = |text: &str, what: &str| -> Result<u64, String> {
            text.parse()
                .map_err(|_| format!("{path}:{}: `{what}` must be a decimal u64, found `{text}`", line_number + 1))
        };
        records.push(Record {
            index: parse_u64(fields[0], "index")?,
            subject: read_field(fields[1]).map_err(|error| format!("{path}:{}: {error}", line_number + 1))?,
            value: parse_u64(fields[2], "value")?,
            tag: read_field(fields[3]).map_err(|error| format!("{path}:{}: {error}", line_number + 1))?,
            salt: read_field(fields[4]).map_err(|error| format!("{path}:{}: {error}", line_number + 1))?,
        });
    }
    if records.is_empty() {
        return Err(format!("{path}: no records"));
    }
    Ok(records)
}

fn build_tree(records: &[Record]) -> Result<(SparseTree, Fr), String> {
    let params = belizechain_zk_attestation::poseidon_config();
    let entries: Vec<(u64, Fr)> = records
        .iter()
        .map(|record| {
            let leaf = belizechain_zk_attestation::leaf_hash(
                &params,
                belizechain_zk_attestation::record_hash(&params, record.subject, Fr::from(record.value), record.tag),
                record.salt,
            );
            (record.index, leaf)
        })
        .collect();
    let tree = SparseTree::new(&params, entries).map_err(|error| error.to_string())?;
    let root = tree.root(&params);
    Ok((tree, root))
}

// --- commands ------------------------------------------------------------

fn cmd_setup(flags: &Flags) -> Result<(), String> {
    let out = flags.get("out")?;
    std::fs::create_dir_all(out).map_err(|error| format!("{out}: {error}"))?;

    // Any shape works for key generation; the witness values are irrelevant to
    // the CRS, only the circuit's structure matters.
    let circuit = AttestationCircuit::empty();
    let setup: Setup =
        belizechain_zk_attestation::setup_for_testing(&circuit).map_err(describe)?;

    std::fs::write(format!("{out}/vk.bin"), &setup.verifying_key)
        .map_err(|error| format!("{out}/vk.bin: {error}"))?;
    std::fs::write(format!("{out}/pk.bin"), &setup.proving_key)
        .map_err(|error| format!("{out}/pk.bin: {error}"))?;

    println!("verifying key: {out}/vk.bin ({} bytes)", setup.verifying_key.len());
    println!("proving key:   {out}/pk.bin ({} bytes)", setup.proving_key.len());
    eprintln!(
        "WARNING: this CRS came from a local, single-party setup. Its toxic waste is\n\
         in this process's memory. Anyone who ran this can forge proofs. Do not use\n\
         it for anything that matters — run a multi-party ceremony."
    );
    Ok(())
}

fn cmd_root(flags: &Flags) -> Result<(), String> {
    let records = load_records(flags.get("records")?)?;
    let (_, root) = build_tree(&records)?;
    println!("{} records", records.len());
    println!("root: 0x{}", hex(&belizechain_zk_attestation::field_to_public_bytes(root)));
    Ok(())
}

fn cmd_prove(flags: &Flags) -> Result<(), String> {
    let records = load_records(flags.get("records")?)?;
    let index = flags.number("index")?;
    let threshold = flags.number("threshold")?;
    let proving_key = read_arg_file(flags, "pk")?;

    let record = records
        .iter()
        .find(|record| record.index == index)
        .ok_or_else(|| format!("no record at index {index}"))?;

    let difference_bits = belizechain_zk_attestation::difference_bits(record.value, threshold)
        .ok_or_else(|| format!("value {} is below threshold {threshold}; no valid proof exists", record.value))?;

    let (tree, root) = build_tree(&records)?;
    let path = tree
        .path(&belizechain_zk_attestation::poseidon_config(), index)
        .map_err(|error| error.to_string())?;

    let params = belizechain_zk_attestation::poseidon_config();
    let record_hash = belizechain_zk_attestation::record_hash(
        &params,
        record.subject,
        Fr::from(record.value),
        record.tag,
    );

    let circuit = AttestationCircuit {
        params: params.clone(),
        set_root: Some(root),
        threshold: Some(Fr::from(threshold)),
        nullifier: Some(belizechain_zk_attestation::tree::hash_two(
            &params,
            record.salt,
            record_hash,
        )),
        subject: Some(record.subject),
        tag: Some(record.tag),
        salt: Some(record.salt),
        value_bits: Some(belizechain_zk_attestation::u64_bits(record.value)),
        diff_bits: Some(difference_bits),
        path: Some(path),
        index_bits: Some(
            (0..belizechain_zk_attestation::TREE_DEPTH)
                .map(|level| (index >> level) & 1 == 1)
                .collect(),
        ),
    };

    // `prove` decodes the proving key with `Validate::Yes`, so a truncated or
    // malformed file fails here rather than after an on-chain rejection.
    let setup = Setup {
        proving_key,
        verifying_key: Vec::new(),
    };
    let material = belizechain_zk_attestation::prove(&circuit, &setup).map_err(describe)?;

    // Self-check before publishing. A proof that does not verify locally is a
    // bug in this pipeline, and an operator deserves to know now.
    let out = flags.get("out")?;
    std::fs::write(out, &material.proof).map_err(|error| format!("{out}: {error}"))?;

    println!("proof: {out} ({} bytes)", material.proof.len());
    print_inputs(&material.inputs);
    eprintln!(
        "note: verify locally with\n  attest verify --vk DIR/vk.bin --proof {out} \\\n    --set-root {} --threshold {threshold} --nullifier {}",
        hex(&material.inputs.set_root),
        hex(&material.inputs.nullifier),
    );
    Ok(())
}

fn cmd_verify(flags: &Flags) -> Result<(), String> {
    let verifying_key = read_arg_file(flags, "vk")?;
    let proof = read_arg_file(flags, "proof")?;

    let inputs = AttestationInputs {
        set_root: parse_hex(flags.get("set-root")?)?,
        threshold: {
            let threshold = flags.number("threshold")?;
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&threshold.to_le_bytes());
            bytes
        },
        nullifier: parse_hex(flags.get("nullifier")?)?,
    };

    // Route through the same checked entry point the pallet uses, so a local pass
    // means an on-chain pass.
    verify_attestation_inputs(&verifying_key, &proof, &inputs)
        .map_err(|error| format!("proof rejected: {error:?}"))?;
    println!("verified");
    print_inputs(&inputs);
    Ok(())
}

// --- helpers -------------------------------------------------------------

fn print_inputs(inputs: &AttestationInputs) {
    println!("set_root:  0x{}", hex(&inputs.set_root));
    println!("threshold: {}", hex(&inputs.threshold));
    println!("nullifier: 0x{}", hex(&inputs.nullifier));
    println!();
    println!(
        "submit with: submit_attestation(set_id, proof_bytes, vec![set_root, threshold, nullifier])"
    );
    println!("value width: {VALUE_BITS} bits");
}

fn describe(error: ProverError) -> String {
    error.to_string()
}

fn read_arg_file(flags: &Flags, name: &str) -> Result<Vec<u8>, String> {
    let path = flags.get(name)?;
    std::fs::read(path).map_err(|error| format!("{path}: {error}"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_hex(text: &str) -> Result<[u8; 32], String> {
    let text = text.strip_prefix("0x").unwrap_or(text);
    if text.len() != 64 {
        return Err(format!("expected 64 hex characters, found {}", text.len()));
    }
    let mut bytes = [0u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| format!("invalid hex at position {}", index * 2))?;
    }
    Ok(bytes)
}

/// Read one witness field: a decimal `u64`, a `0x`-prefixed hex `u64`, or a full
/// 64-character hex field element.
fn read_field(text: &str) -> Result<Fr, String> {
    let trimmed = text.trim();
    let as_u64 = if let Some(hex) = trimmed.strip_prefix("0x") {
        if hex.len() == 64 {
            // A full-width element; the decimal path would overflow.
            return belizechain_zk_attestation::public_bytes_to_field(&parse_hex(hex)?)
                .ok_or_else(|| format!("`{trimmed}` is not a canonical field element"));
        }
        u64::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<u64>().ok()
    };
    as_u64
        .map(Fr::from)
        .ok_or_else(|| format!("`{trimmed}` is not a decimal u64 or 0x-hex u64"))
}
