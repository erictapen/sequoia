/// Decrypts asymmetrically-encrypted OpenPGP messages using the
/// openpgp crate, Sequoia's low-level API.

use std::collections::HashMap;
use std::env;
use std::io;

use anyhow::Context;

use sequoia_openpgp as openpgp;

use crate::openpgp::cert::prelude::*;
use crate::openpgp::crypto::{KeyPair, SessionKey};
use crate::openpgp::types::SymmetricAlgorithm;
use crate::openpgp::parse::{
    Parse,
    stream::{
        DecryptionHelper,
        DecryptorBuilder,
        VerificationHelper,
        GoodChecksum,
        MessageStructure,
        MessageLayer,
    },
};
use crate::openpgp::policy::Policy;
use crate::openpgp::policy::StandardPolicy as P;

pub fn main() -> openpgp::Result<()> {
    let p = &P::new();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("A simple decryption filter.\n\n\
                Usage: {} <keyfile> [<keyfile>...] <input >output\n", args[0]));
    }

    // Read the transferable secret keys from the given files.
    let certs =
        args[1..].iter().map(|f| {
            openpgp::Cert::from_file(f)
        }).collect::<openpgp::Result<Vec<_>>>()
        .context("Failed to read key")?;

    // Now, create a decryptor with a helper using the given Certs.
    let mut decryptor =
        DecryptorBuilder::from_reader(io::stdin()).unwrap()
        .with_policy(p, None, Helper::new(p, certs))?;

    // Finally, stream the decrypted data to stdout.
    io::copy(&mut decryptor, &mut io::stdout())
        .context("Decryption failed")?;

    Ok(())
}

/// This helper provides secrets for the decryption, fetches public
/// keys for the signature verification and implements the
/// verification policy.
struct Helper {
    keys: HashMap<openpgp::KeyID, KeyPair>,
}

impl Helper {
    /// Creates a Helper for the given Certs with appropriate secrets.
    fn new(p: &dyn Policy, certs: Vec<openpgp::Cert>) -> Self {
        // Map (sub)KeyIDs to secrets.
        let mut keys = HashMap::new();
        for cert in certs {
            for ka in cert.keys().unencrypted_secret().with_policy(p, None)
                .for_storage_encryption().for_transport_encryption()
            {
                keys.insert(ka.key().keyid(),
                            ka.key().clone().into_keypair().unwrap());
            }
        }

        Helper {
            keys,
        }
    }
}

impl DecryptionHelper for Helper {
    fn decrypt<D>(&mut self,
                  pkesks: &[openpgp::packet::PKESK],
                  _skesks: &[openpgp::packet::SKESK],
                  sym_algo: Option<SymmetricAlgorithm>,
                  mut decrypt: D)
                  -> openpgp::Result<Option<openpgp::Fingerprint>>
        where D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool
    {
        // Try each PKESK until we succeed.
        for pkesk in pkesks {
            if let Some(pair) = self.keys.get_mut(pkesk.recipient()) {
                if pkesk.decrypt(pair, sym_algo)
                    .map(|(algo, session_key)| decrypt(algo, &session_key))
                    .unwrap_or(false)
                {
                    break;
                }
            }
        }
        // XXX: In production code, return the Fingerprint of the
        // recipient's Cert here
        Ok(None)
    }
}

impl VerificationHelper for Helper {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle])
                       -> openpgp::Result<Vec<openpgp::Cert>> {
        Ok(Vec::new()) // Feed the Certs to the verifier here.
    }
    fn check(&mut self, structure: MessageStructure)
             -> openpgp::Result<()> {
        for layer in structure.iter() {
            match layer {
                MessageLayer::Compression { algo } =>
                    eprintln!("Compressed using {}", algo),
                MessageLayer::Encryption { sym_algo, aead_algo } =>
                    if let Some(aead_algo) = aead_algo {
                        eprintln!("Encrypted and protected using {}/{}",
                                  sym_algo, aead_algo);
                    } else {
                        eprintln!("Encrypted using {}", sym_algo);
                    },
                MessageLayer::SignatureGroup { ref results } =>
                    for result in results {
                        match result {
                            Ok(GoodChecksum { ka, .. }) => {
                                eprintln!("Good signature from {}", ka.cert());
                            },
                            Err(e) =>
                                eprintln!("Error: {:?}", e),
                        }
                    }
            }
        }
        Ok(()) // Implement your verification policy here.
    }
}
