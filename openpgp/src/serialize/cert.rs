use crate::Result;
use crate::cert::prelude::*;
use crate::packet::{header::BodyLength, key, Signature, Tag};
use crate::seal;
use crate::serialize::{
    PacketRef,
    Marshal, MarshalInto,
    generic_serialize_into, generic_export_into,
};


impl Cert {
    /// Serializes or exports the Cert.
    ///
    /// If `export` is true, then non-exportable signatures are not
    /// written, and components without any exportable binding
    /// signature or revocation are not exported.
    ///
    /// The signatures are ordered from authenticated and most
    /// important to not authenticated and most likely to be abused.
    /// The order is:
    ///
    ///   - Self revocations first.  They are authenticated and the
    ///     most important information.
    ///   - Self signatures.  They are authenticated.
    ///   - Other signatures.  They are not authenticated at this point.
    ///   - Other revocations.  They are not authenticated, and likely
    ///     not well supported in other implementations, hence the
    ///     least reliable way of revoking keys and therefore least
    ///     useful and most likely to be abused.
    fn serialize_common(&self, o: &mut dyn std::io::Write, export: bool)
                        -> Result<()>
    {
        let primary = self.primary_key();
        PacketRef::PublicKey(primary.key())
            .serialize(o)?;

        // Writes a signature if it is exportable or `! export`.
        let serialize_sig =
            |o: &mut dyn std::io::Write, sig: &Signature| -> Result<()>
        {
            if export {
                if sig.exportable().is_ok() {
                    PacketRef::Signature(sig).export(o)?;
                }
            } else {
                PacketRef::Signature(sig).serialize(o)?;
            }
            Ok(())
        };

        for s in primary.self_revocations() {
            serialize_sig(o, s)?;
        }
        for s in primary.self_signatures() {
            serialize_sig(o, s)?;
        }
        for s in primary.certifications() {
            serialize_sig(o, s)?;
        }
        for s in primary.other_revocations() {
            serialize_sig(o, s)?;
        }

        for u in self.userids() {
            if export && ! u.self_signatures().iter().chain(u.self_revocations()).any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            PacketRef::UserID(u.userid()).serialize(o)?;
            for s in u.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in u.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in u.certifications() {
                serialize_sig(o, s)?;
            }
            for s in u.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for u in self.user_attributes() {
            if export && ! u.self_signatures().iter().chain(u.self_revocations()).any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            PacketRef::UserAttribute(u.user_attribute()).serialize(o)?;
            for s in u.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in u.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in u.certifications() {
                serialize_sig(o, s)?;
            }
            for s in u.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for k in self.subkeys() {
            if export && ! k.self_signatures().iter().chain(k.self_revocations()).any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            PacketRef::PublicSubkey(k.key()).serialize(o)?;
            for s in k.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in k.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in k.certifications() {
                serialize_sig(o, s)?;
            }
            for s in k.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for u in self.unknowns() {
            if export && ! u.certifications().iter().any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            PacketRef::Unknown(u.unknown()).serialize(o)?;

            for s in u.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in u.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in u.certifications() {
                serialize_sig(o, s)?;
            }
            for s in u.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for s in self.bad_signatures() {
            serialize_sig(o, s)?;
        }

        Ok(())
    }
}

impl crate::serialize::Serialize for Cert {}

impl seal::Sealed for Cert {}
impl Marshal for Cert {
    fn serialize(&self, o: &mut dyn std::io::Write) -> Result<()> {
        self.serialize_common(o, false)
    }

    fn export(&self, o: &mut dyn std::io::Write) -> Result<()> {
        self.serialize_common(o, true)
    }
}

impl crate::serialize::SerializeInto for Cert {}

impl MarshalInto for Cert {
    fn serialized_len(&self) -> usize {
        let mut l = 0;
        let primary = self.primary_key();
        l += PacketRef::PublicKey(primary.key()).serialized_len();

        for s in primary.self_revocations() {
            l += PacketRef::Signature(s).serialized_len();
        }
        for s in primary.self_signatures() {
            l += PacketRef::Signature(s).serialized_len();
        }
        for s in primary.certifications() {
            l += PacketRef::Signature(s).serialized_len();
        }
        for s in primary.other_revocations() {
            l += PacketRef::Signature(s).serialized_len();
        }

        for u in self.userids() {
            l += PacketRef::UserID(u.userid()).serialized_len();

            for s in u.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for u in self.user_attributes() {
            l += PacketRef::UserAttribute(u.user_attribute()).serialized_len();

            for s in u.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for k in self.subkeys() {
            l += PacketRef::PublicSubkey(k.key()).serialized_len();

            for s in k.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in k.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in k.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in k.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for u in self.unknowns() {
            l += PacketRef::Unknown(u.unknown()).serialized_len();

            for s in u.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for s in self.bad_signatures() {
            l += PacketRef::Signature(s).serialized_len();
        }

        l
    }

    fn serialize_into(&self, buf: &mut [u8]) -> Result<usize> {
        generic_serialize_into(self, self.serialized_len(), buf)
    }

    fn export_into(&self, buf: &mut [u8]) -> Result<usize> {
        generic_export_into(self, self.serialized_len(), buf)
    }
}

impl Cert {
    /// Derive a [`TSK`] object from this key.
    ///
    /// This object writes out secret keys during serialization.
    ///
    /// [`TSK`]: ../serialize/struct.TSK.html
    pub fn as_tsk<'a>(&'a self) -> TSK<'a> {
        TSK::new(self)
    }
}

/// A reference to a `Cert` that allows serialization of secret keys.
///
/// To avoid accidental leakage, secret keys are not serialized when a
/// serializing a [`Cert`].  To serialize [`Cert`]s with secret keys,
/// use [`Cert::as_tsk()`] to create a `TSK`, which is a shim on top
/// of the `Cert`, and serialize this.
///
/// [`Cert`]: ../cert/struct.Cert.html
/// [`Cert::as_tsk()`]: ../cert/struct.Cert.html#method.as_tsk
///
/// # Examples
///
/// ```
/// # use sequoia_openpgp::{*, cert::*, parse::Parse, serialize::Serialize};
/// # fn main() -> Result<()> {
/// let (cert, _) = CertBuilder::new().generate()?;
/// assert!(cert.is_tsk());
///
/// let mut buf = Vec::new();
/// cert.as_tsk().serialize(&mut buf)?;
///
/// let cert_ = Cert::from_bytes(&buf)?;
/// assert!(cert_.is_tsk());
/// assert_eq!(cert, cert_);
/// # Ok(()) }
/// ```
pub struct TSK<'a> {
    cert: &'a Cert,
    filter: Option<Box<dyn Fn(&'a key::UnspecifiedSecret) -> bool + 'a>>,
    emit_stubs: bool,
}

impl<'a> TSK<'a> {
    /// Creates a new view for the given `Cert`.
    fn new(cert: &'a Cert) -> Self {
        Self {
            cert,
            filter: None,
            emit_stubs: false,
        }
    }

    /// Filters which secret keys to export using the given predicate.
    ///
    /// Note that the given filter replaces any existing filter.
    ///
    /// # Examples
    ///
    /// This example demonstrates how to create a TSK with a detached
    /// primary secret key.
    ///
    /// ```
    /// # use sequoia_openpgp::{*, cert::*, parse::Parse, serialize::Serialize};
    /// use sequoia_openpgp::policy::StandardPolicy;
    ///
    /// # fn main() -> Result<()> {
    /// let p = &StandardPolicy::new();
    ///
    /// let (cert, _) = CertBuilder::new().add_signing_subkey().generate()?;
    /// assert_eq!(cert.keys().with_policy(p, None).alive().revoked(false).secret().count(), 2);
    ///
    /// // Only write out the subkey's secret.
    /// let mut buf = Vec::new();
    /// cert.as_tsk()
    ///     .set_filter(|k| k.fingerprint() != cert.fingerprint())
    ///     .serialize(&mut buf)?;
    ///
    /// let cert_ = Cert::from_bytes(&buf)?;
    /// assert!(! cert_.primary_key().has_secret());
    /// assert_eq!(cert_.keys().with_policy(p, None).alive().revoked(false).secret().count(), 1);
    /// # Ok(()) }
    /// ```
    pub fn set_filter<P>(mut self, predicate: P) -> Self
        where P: 'a + Fn(&'a key::UnspecifiedSecret) -> bool
    {
        self.filter = Some(Box::new(predicate));
        self
    }

    /// Changes `TSK` to emit secret key stubs.
    ///
    /// If [`TSK::set_filter`] is used to selectively export secret
    /// keys, or if the cert contains both keys without secret key
    /// material and with secret key material, then are two ways to
    /// serialize this cert.  Neither is sanctioned by the OpenPGP
    /// standard.
    ///
    /// The default way is to simply emit public key packets when no
    /// secret key material is available.  While straight forward,
    /// this may be in violation of [Section 11.2 of RFC 4880].
    ///
    /// The alternative is to emit a secret key packet with a
    /// placeholder secret key value.  GnuPG uses this variant with a
    /// private [`S2K`] format.  If interoperability with GnuPG is a
    /// concern, use this variant.
    ///
    /// See [this test] for support in other implementations.
    ///
    ///   [`TSK::set_filter`]: #method.set_filter
    ///   [Section 11.2 of RFC 4880]: https://tools.ietf.org/html/rfc4880#section-11.2
    ///   [`S2K`]: ../crypto/enum.S2K.html
    ///   [this test]: https://tests.sequoia-pgp.org/#Detached_primary_key
    ///
    /// # Examples
    ///
    /// This example demonstrates how to create a TSK with a detached
    /// primary secret key, serializing it using secret key stubs.
    ///
    /// ```
    /// # fn main() -> sequoia_openpgp::Result<()> {
    /// # use std::convert::TryFrom;
    /// use sequoia_openpgp as openpgp;
    /// use openpgp::packet::key::*;
    /// # use openpgp::{types::*, crypto::S2K};
    /// # use openpgp::{*, cert::*, parse::Parse, serialize::Serialize};
    ///
    /// let p = &openpgp::policy::StandardPolicy::new();
    ///
    /// let (cert, _) = CertBuilder::new().add_signing_subkey().generate()?;
    /// assert_eq!(cert.keys().with_policy(p, None)
    ///            .alive().revoked(false).unencrypted_secret().count(), 2);
    ///
    /// // Only write out the subkey's secret, the primary key is "detached".
    /// let mut buf = Vec::new();
    /// cert.as_tsk()
    ///     .set_filter(|k| k.fingerprint() != cert.fingerprint())
    ///     .emit_secret_key_stubs(true)
    ///     .serialize(&mut buf)?;
    ///
    /// # let pp = PacketPile::from_bytes(&buf)?;
    /// # assert_eq!(pp.path_ref(&[0]).unwrap().kind(),
    /// #            Some(packet::Tag::SecretKey));
    /// let cert_ = Cert::from_bytes(&buf)?;
    /// // The primary key has an "encrypted" stub.
    /// assert!(cert_.primary_key().has_secret());
    /// assert_eq!(cert_.keys().with_policy(p, None)
    ///            .alive().revoked(false).unencrypted_secret().count(), 1);
    /// # if let Some(SecretKeyMaterial::Encrypted(sec)) =
    /// #     cert_.primary_key().optional_secret()
    /// # {
    /// #     assert_eq!(sec.algo(), SymmetricAlgorithm::Unencrypted);
    /// #     if let S2K::Private { tag, .. } = sec.s2k() {
    /// #         assert_eq!(*tag, 101);
    /// #     } else {
    /// #         panic!("expected proprietary S2K type");
    /// #     }
    /// # } else {
    /// #     panic!("expected ''encrypted'' secret key stub");
    /// # }
    /// # Ok(()) }
    /// ```
    pub fn emit_secret_key_stubs(mut self, emit_stubs: bool) -> Self {
        self.emit_stubs = emit_stubs;
        self
    }

    /// Serializes or exports the Cert.
    ///
    /// If `export` is true, then non-exportable signatures are not
    /// written, and components without any exportable binding
    /// signature or revocation are not exported.
    fn serialize_common(&self, o: &mut dyn std::io::Write, export: bool)
                        -> Result<()>
    {
        // Writes a signature if it is exportable or `! export`.
        let serialize_sig =
            |o: &mut dyn std::io::Write, sig: &Signature| -> Result<()>
        {
            if export {
                if sig.exportable().is_ok() {
                    PacketRef::Signature(sig).export(o)?;
                }
            } else {
                PacketRef::Signature(sig).serialize(o)?;
            }
            Ok(())
        };

        // Serializes public or secret key depending on the filter.
        let serialize_key =
            |o: &mut dyn std::io::Write, key: &'a key::UnspecifiedSecret,
             tag_public, tag_secret|
        {
            let tag = if key.has_secret()
                && self.filter.as_ref().map(|f| f(key)).unwrap_or(true) {
                tag_secret
            } else {
                tag_public
            };

            if self.emit_stubs && (tag == Tag::PublicKey
                                   || tag == Tag::PublicSubkey) {
                // Emit a GnuPG-style secret key stub.
                let stub = crate::crypto::S2K::Private {
                    tag: 101,
                    parameters: Some(vec![
                        0,    // "hash algo"
                        0x47, // 'G'
                        0x4e, // 'N'
                        0x55, // 'U'
                        1     // "mode"
                    ].into()),
                };
                let key_with_stub = key.clone()
                    .add_secret(key::SecretKeyMaterial::Encrypted(
                        key::Encrypted::new(
                            stub, 0.into(),
                            // Mirrors more closely what GnuPG 2.1
                            // does (oddly, GnuPG 1.4 emits 0xfe
                            // here).
                            Some(crate::crypto::mpi::SecretKeyChecksum::Sum16),
                            vec![].into()))).0;
                return match tag {
                    Tag::PublicKey =>
                        crate::Packet::SecretKey(key_with_stub.into())
                            .serialize(o),
                    Tag::PublicSubkey =>
                        crate::Packet::SecretSubkey(key_with_stub.into())
                            .serialize(o),
                    _ => unreachable!(),
                };
            }

            match tag {
                Tag::PublicKey =>
                    PacketRef::PublicKey(key.into()).serialize(o),
                Tag::PublicSubkey =>
                    PacketRef::PublicSubkey(key.into()).serialize(o),
                Tag::SecretKey =>
                    PacketRef::SecretKey(key.into()).serialize(o),
                Tag::SecretSubkey =>
                    PacketRef::SecretSubkey(key.into()).serialize(o),
                _ => unreachable!(),
            }
        };

        let primary = self.cert.primary_key();
        serialize_key(o, primary.key().into(),
                      Tag::PublicKey, Tag::SecretKey)?;

        for s in primary.self_signatures() {
            serialize_sig(o, s)?;
        }
        for s in primary.self_revocations() {
            serialize_sig(o, s)?;
        }
        for s in primary.certifications() {
            serialize_sig(o, s)?;
        }
        for s in primary.other_revocations() {
            serialize_sig(o, s)?;
        }

        for u in self.cert.userids() {
            if export && ! u.self_signatures().iter().chain(u.self_revocations()).any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            PacketRef::UserID(u.userid()).serialize(o)?;
            for s in u.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in u.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in u.certifications() {
                serialize_sig(o, s)?;
            }
            for s in u.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for u in self.cert.user_attributes() {
            if export && ! u.self_signatures().iter().chain(u.self_revocations()).any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            PacketRef::UserAttribute(u.user_attribute()).serialize(o)?;
            for s in u.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in u.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in u.certifications() {
                serialize_sig(o, s)?;
            }
            for s in u.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for k in self.cert.subkeys() {
            if export && ! k.self_signatures().iter().chain(k.self_revocations()).any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            serialize_key(o, k.key().into(),
                          Tag::PublicSubkey, Tag::SecretSubkey)?;
            for s in k.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in k.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in k.certifications() {
                serialize_sig(o, s)?;
            }
            for s in k.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for u in self.cert.unknowns() {
            if export && ! u.certifications().iter().any(
                |s| s.exportable().is_ok())
            {
                // No exportable selfsig on this component, skip it.
                continue;
            }

            PacketRef::Unknown(&u.unknown()).serialize(o)?;

            for s in u.self_revocations() {
                serialize_sig(o, s)?;
            }
            for s in u.self_signatures() {
                serialize_sig(o, s)?;
            }
            for s in u.certifications() {
                serialize_sig(o, s)?;
            }
            for s in u.other_revocations() {
                serialize_sig(o, s)?;
            }
        }

        for s in self.cert.bad_signatures() {
            serialize_sig(o, s)?;
        }

        Ok(())
    }
}

impl<'a> crate::serialize::Serialize for TSK<'a> {}

impl<'a> seal::Sealed for TSK<'a> {}
impl<'a> Marshal for TSK<'a> {
    fn serialize(&self, o: &mut dyn std::io::Write) -> Result<()> {
        self.serialize_common(o, false)
    }

    fn export(&self, o: &mut dyn std::io::Write) -> Result<()> {
        self.serialize_common(o, true)
    }
}

impl<'a> crate::serialize::SerializeInto for TSK<'a> {}

impl<'a> MarshalInto for TSK<'a> {
    fn serialized_len(&self) -> usize {
        let mut l = 0;

        // Serializes public or secret key depending on the filter.
        let serialized_len_key
            = |key: &'a key::UnspecifiedSecret, tag_public, tag_secret|
        {
            let tag = if key.has_secret()
                && self.filter.as_ref().map(|f| f(key)).unwrap_or(true) {
                tag_secret
            } else {
                tag_public
            };

            if self.emit_stubs && (tag == Tag::PublicKey
                                   || tag == Tag::PublicSubkey) {
                // Emit a GnuPG-style secret key stub.  The stub
                // extends the public key by 8 bytes.
                let l = key.net_len_key(false) + 8;
                return 1 // CTB
                    + BodyLength::Full(l as u32).serialized_len()
                    + l;
            }

            let packet = match tag {
                Tag::PublicKey => PacketRef::PublicKey(key.into()),
                Tag::PublicSubkey => PacketRef::PublicSubkey(key.into()),
                Tag::SecretKey => PacketRef::SecretKey(key.into()),
                Tag::SecretSubkey => PacketRef::SecretSubkey(key.into()),
                _ => unreachable!(),
            };

            packet.serialized_len()
        };

        let primary = self.cert.primary_key();
        l += serialized_len_key(primary.key().into(),
                                Tag::PublicKey, Tag::SecretKey);

        for s in primary.self_signatures() {
            l += PacketRef::Signature(s).serialized_len();
        }
        for s in primary.self_revocations() {
            l += PacketRef::Signature(s).serialized_len();
        }
        for s in primary.other_revocations() {
            l += PacketRef::Signature(s).serialized_len();
        }
        for s in primary.certifications() {
            l += PacketRef::Signature(s).serialized_len();
        }

        for u in self.cert.userids() {
            l += PacketRef::UserID(u.userid()).serialized_len();

            for s in u.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for u in self.cert.user_attributes() {
            l += PacketRef::UserAttribute(u.user_attribute()).serialized_len();

            for s in u.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for k in self.cert.subkeys() {
            l += serialized_len_key(k.key().into(),
                                    Tag::PublicSubkey, Tag::SecretSubkey);

            for s in k.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in k.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in k.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in k.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for u in self.cert.unknowns() {
            l += PacketRef::Unknown(u.unknown()).serialized_len();

            for s in u.self_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.self_signatures() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.other_revocations() {
                l += PacketRef::Signature(s).serialized_len();
            }
            for s in u.certifications() {
                l += PacketRef::Signature(s).serialized_len();
            }
        }

        for s in self.cert.bad_signatures() {
            l += PacketRef::Signature(s).serialized_len();
        }

        l
    }

    fn serialize_into(&self, buf: &mut [u8]) -> Result<usize> {
        generic_serialize_into(self, self.serialized_len(), buf)
    }

    fn export_into(&self, buf: &mut [u8]) -> Result<usize> {
        generic_export_into(self, self.serialized_len(), buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::vec_truncate;
    use crate::parse::Parse;
    use crate::packet::key;
    use crate::policy::StandardPolicy as P;

    /// Demonstrates that public keys and all components are
    /// serialized.
    #[test]
    fn roundtrip_cert() {
        for test in crate::tests::CERTS {
            let cert = match Cert::from_bytes(test.bytes) {
                Ok(t) => t,
                Err(_) => continue,
            };
            assert!(! cert.is_tsk());
            let buf = cert.as_tsk().to_vec().unwrap();
            let cert_ = Cert::from_bytes(&buf).unwrap();

            assert_eq!(cert, cert_, "roundtripping {}.pgp failed", test);
        }
    }

    /// Demonstrates that secret keys and all components are
    /// serialized.
    #[test]
    fn roundtrip_tsk() {
        for test in crate::tests::TSKS {
            let cert = Cert::from_bytes(test.bytes).unwrap();
            assert!(cert.is_tsk());

            let mut buf = Vec::new();
            cert.as_tsk().serialize(&mut buf).unwrap();
            let cert_ = Cert::from_bytes(&buf).unwrap();

            assert_eq!(cert, cert_, "roundtripping {}-private.pgp failed", test);

            // This time, use a trivial filter.
            let mut buf = Vec::new();
            cert.as_tsk().set_filter(|_| true).serialize(&mut buf).unwrap();
            let cert_ = Cert::from_bytes(&buf).unwrap();

            assert_eq!(cert, cert_, "roundtripping {}-private.pgp failed", test);
        }
    }

    /// Demonstrates that TSK::serialize() with the right filter
    /// reduces to Cert::serialize().
    #[test]
    fn reduce_to_cert_serialize() {
        for test in crate::tests::TSKS {
            let cert = Cert::from_bytes(test.bytes).unwrap();
            assert!(cert.is_tsk());

            // First, use Cert::serialize().
            let mut buf_cert = Vec::new();
            cert.serialize(&mut buf_cert).unwrap();

            // When serializing using TSK::serialize, filter out all
            // secret keys.
            let mut buf_tsk = Vec::new();
            cert.as_tsk().set_filter(|_| false).serialize(&mut buf_tsk).unwrap();

            // Check for equality.
            let cert_ = Cert::from_bytes(&buf_cert).unwrap();
            let tsk_ = Cert::from_bytes(&buf_tsk).unwrap();
            assert_eq!(cert_, tsk_,
                       "reducing failed on {}-private.pgp: not Cert::eq",
                       test);

            // Check for identinty.
            assert_eq!(buf_cert, buf_tsk,
                       "reducing failed on {}-private.pgp: serialized identity",
                       test);
        }
    }

    #[test]
    fn export() {
        use crate::Packet;
        use crate::cert::prelude::*;
        use crate::types::{Curve, KeyFlags, SignatureType};
        use crate::packet::{
            signature, UserID, user_attribute::{UserAttribute, Subpacket},
            key::Key4,
        };

        let p = &P::new();

        let (cert, _) = CertBuilder::new().generate().unwrap();
        let mut keypair = cert.primary_key().key().clone().parts_into_secret()
            .unwrap().into_keypair().unwrap();

        let key: key::SecretSubkey =
            Key4::generate_ecc(false, Curve::Cv25519).unwrap().into();
        let key_binding = key.bind(
            &mut keypair, &cert,
            signature::SignatureBuilder::new(SignatureType::SubkeyBinding)
                .set_key_flags(
                    KeyFlags::empty().set_transport_encryption())
                .unwrap()
                .set_exportable_certification(false).unwrap()).unwrap();

        let uid = UserID::from("foo");
        let uid_binding = uid.bind(
            &mut keypair, &cert,
            signature::SignatureBuilder::from(
                cert.primary_key().with_policy(p, None).unwrap()
                    .direct_key_signature().unwrap().clone())
                    .set_type(SignatureType::PositiveCertification)
                    .preserve_signature_creation_time().unwrap()
                    .set_exportable_certification(false).unwrap()).unwrap();

        let ua = UserAttribute::new(&[
            Subpacket::Unknown(2, b"foo".to_vec().into_boxed_slice()),
        ]).unwrap();
        let ua_binding = ua.bind(
            &mut keypair, &cert,
            signature::SignatureBuilder::from(
                cert.primary_key().with_policy(p, None).unwrap()
                    .direct_key_signature().unwrap().clone())
                .set_type(SignatureType::PositiveCertification)
                .preserve_signature_creation_time().unwrap()
                .set_exportable_certification(false).unwrap()).unwrap();

        let cert = cert.insert_packets(vec![
            Packet::SecretSubkey(key), key_binding.into(),
            uid.into(), uid_binding.into(),
            ua.into(), ua_binding.into(),
        ]).unwrap();

        assert_eq!(cert.subkeys().count(), 1);
        cert.subkeys().nth(0).unwrap().binding_signature(p, None).unwrap();
        assert_eq!(cert.userids().count(), 1);
        assert!(cert.userids().with_policy(p, None).nth(0).is_some());
        assert_eq!(cert.user_attributes().count(), 1);
        assert!(cert.user_attributes().with_policy(p, None).nth(0).is_some());

        // The binding signature is not exportable, so when we export
        // and re-parse, we expect the userid to be gone.
        let mut buf = Vec::new();
        cert.export(&mut buf).unwrap();
        let cert_ = Cert::from_bytes(&buf).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        let mut buf = vec![0; cert.serialized_len()];
        let l = cert.export_into(&mut buf).unwrap();
        vec_truncate(&mut buf, l);
        let cert_ = Cert::from_bytes(&buf).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        let cert_ = Cert::from_bytes(&cert.export_to_vec().unwrap()).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        // Same, this time using the armor encoder.
        let mut buf = Vec::new();
        cert.armored().export(&mut buf).unwrap();
        let cert_ = Cert::from_bytes(&buf).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        let mut buf = vec![0; cert.serialized_len()];
        let l = cert.armored().export_into(&mut buf).unwrap();
        vec_truncate(&mut buf, l);
        let cert_ = Cert::from_bytes(&buf).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        let cert_ =
            Cert::from_bytes(&cert.armored().export_to_vec().unwrap()).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        // Same, this time as TSKs.
        let mut buf = Vec::new();
        cert.as_tsk().export(&mut buf).unwrap();
        let cert_ = Cert::from_bytes(&buf).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        let mut buf = vec![0; cert.serialized_len()];
        let l = cert.as_tsk().export_into(&mut buf).unwrap();
        vec_truncate(&mut buf, l);
        let cert_ = Cert::from_bytes(&buf).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);

        let cert_ =
            Cert::from_bytes(&cert.as_tsk().export_to_vec().unwrap()).unwrap();
        assert_eq!(cert_.subkeys().count(), 0);
        assert_eq!(cert_.userids().count(), 0);
        assert_eq!(cert_.user_attributes().count(), 0);
    }

    /// Tests that GnuPG-style stubs are preserved when roundtripping.
    #[test]
    fn issue_613() -> Result<()> {
        use crate::packet::key::*;
        use crate::{types::*, crypto::S2K};
        use crate::{*, cert::*, parse::Parse};
        let p = &crate::policy::StandardPolicy::new();

        let (cert, _) = CertBuilder::new().add_signing_subkey().generate()?;
        assert_eq!(cert.keys().with_policy(p, None)
                   .alive().revoked(false).unencrypted_secret().count(), 2);

        // Only write out the subkey's secret, the primary key is "detached".
        let buf = cert.as_tsk()
            .set_filter(|k| k.fingerprint() != cert.fingerprint())
            .emit_secret_key_stubs(true)
            .to_vec()?;

        // Try parsing it.
        let cert_ = Cert::from_bytes(&buf)?;

        // The primary key has an "encrypted" stub.
        assert!(cert_.primary_key().has_secret());
        assert_eq!(cert_.keys().with_policy(p, None)
                   .alive().revoked(false).unencrypted_secret().count(), 1);
        if let Some(SecretKeyMaterial::Encrypted(sec)) =
            cert_.primary_key().optional_secret()
        {
            assert_eq!(sec.algo(), SymmetricAlgorithm::Unencrypted);
            if let S2K::Private { tag, .. } = sec.s2k() {
                assert_eq!(*tag, 101);
            } else {
                panic!("expected proprietary S2K type");
            }
        } else {
            panic!("expected ''encrypted'' secret key stub");
        }

        // When roundtripping such a key, the stub should be preserved.
        let buf_ = cert_.as_tsk().to_vec()?;
        assert_eq!(buf, buf_);
        Ok(())
    }
}
