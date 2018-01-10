use std::io;
use std::cmp;

use super::*;

mod partial_body;
use self::partial_body::PartialBodyFilter;

// Whether to trace the modules execution (on stderr).
const TRACE : bool = false;

#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
fn path_to(artifact: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "data", "messages", artifact]
        .iter().collect()
}

fn write_byte<W: io::Write>(o: &mut W, b: u8) -> Result<(), io::Error> {
    let b : [u8; 1] = [b; 1];
    o.write_all(&b[..])
}

fn write_be_u16<W: io::Write>(o: &mut W, n: u16) -> Result<(), io::Error> {
    let b : [u8; 2] = [ ((n >> 8) & 0xFF) as u8, (n & 0xFF) as u8 ];
    o.write_all(&b[..])
}

fn write_be_u32<W: io::Write>(o: &mut W, n: u32) -> Result<(), io::Error> {
    let b : [u8; 4] = [ (n >> 24) as u8, ((n >> 16) & 0xFF) as u8,
                         ((n >> 8) & 0xFF) as u8, (n & 0xFF) as u8 ];
    o.write_all(&b[..])
}

/// Returns a new format body length header appropriate for the given
/// body length.
///
/// Note: the returned byte stream does not include a ctb header.
pub fn body_length_new_format(l: BodyLength) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(5);
    match l {
        BodyLength::Full(l) => {
            if l <= 191 {
                // Writing to a Vec can never fail.
                write_byte(&mut buffer, l as u8).unwrap();
            } else if l < 8383 {
                let v = l - 192;
                let v = v + (192 << 8);
                write_be_u16(&mut buffer, v as u16).unwrap();
            } else {
                write_be_u32(&mut buffer, l).unwrap();
            }
        },
        BodyLength::Partial(_) => {
            unimplemented!();
        },
        BodyLength::Indeterminate => {
            panic!("BodyLength::Indeterminate not supported by new format packets.");
        },
    }

    buffer
}

/// Returns an old format body length header appropriate for the given
/// body length.
///
/// Note: the returned byte stream does not include a ctb header.
pub fn body_length_old_format(l: BodyLength) -> Vec<u8> {
    // Assume an optimal encoding is desired.
    let mut buffer = Vec::with_capacity(4);
    match l {
        BodyLength::Full(l) => {
            match l {
                // One octet length.
                // write_byte can't fail for a Vec.
                0 ... 0xFF =>
                    write_byte(&mut buffer, l as u8).unwrap(),
                // Two octet length.
                0x1_00 ... 0xFF_FF =>
                    write_be_u16(&mut buffer, l as u16).unwrap(),
                // Four octet length,
                _ =>
                    write_be_u32(&mut buffer, l as u32).unwrap(),
            }
        },
        BodyLength::Indeterminate => {},
        BodyLength::Partial(_) =>
            panic!("old format CTB don't supported partial body length encoding."),
    }

    buffer
}

/// Returns a new format CTB.
pub fn ctb_new(tag: Tag) -> u8 {
    0b1100_0000u8 | Tag::to_numeric(tag)
}

/// Returns an old format CTB.
pub fn ctb_old(tag: Tag, l: BodyLength) -> u8 {
    let len_encoding : u8 = match l {
        // Assume an optimal encoding.
        BodyLength::Full(l) => {
            match l {
                // One octet length.
                0 ... 0xFF => 0,
                // Two octet length.
                0x1_00 ... 0xFF_FF => 1,
                // Four octet length,
                _ => 2,
            }
        },
        BodyLength::Partial(_) => {
            panic!("Partial body lengths are not support for old format packets.");
        },
        BodyLength::Indeterminate => 3,
    };
    assert!(len_encoding <= 3);

    let tag = Tag::to_numeric(tag);
    // Only tags 0-15 are supported.
    assert!(tag < 16);

    0b1000_0000u8 | (tag << 2) | len_encoding
}

impl Unknown {
    /// Writes a serialized version of the specified `Unknown` packet
    /// to `o`.
    pub fn serialize<W: io::Write>(&self, o: &mut W)
            -> Result<(), io::Error> {
        let body = if let Some(ref body) = self.common.body {
            &body[..]
        } else {
            &b""[..]
        };

        write_byte(o, ctb_new(self.tag))?;
        o.write_all(&body_length_new_format(
            BodyLength::Full(body.len() as u32))[..])?;
        o.write_all(&body[..])?;

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(4096);
        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o
    }
}

impl Signature {
    /// Writes a serialized version of the specified `Signature`
    /// packet to `o`.
    ///
    /// Note: this function does not computer the signature (which
    /// would require access to the private key); it assumes that
    /// sig.mpis is up to date.
    pub fn serialize<W: io::Write>(&self, o: &mut W)
            -> Result<(), io::Error> {
        let len = 1 // version
            + 1 // signature type.
            + 1 // pk algorithm
            + 1 // hash algorithm
            + 2 // hashed area size
            + self.hashed_area.len()
            + 2 // unhashed area size
            + self.unhashed_area.len()
            + 2 // hash prefix
            + self.mpis.len();

        write_byte(o, ctb_new(Tag::Signature))?;
        o.write_all(
            &body_length_new_format(BodyLength::Full(len as u32))[..])?;

        // XXX: Return an error.
        assert_eq!(self.version, 4);
        write_byte(o, self.version)?;
        write_byte(o, self.sigtype)?;
        write_byte(o, self.pk_algo)?;
        write_byte(o, self.hash_algo)?;

        // XXX: Return an error.
        assert!(self.hashed_area.len() <= std::u16::MAX as usize);
        write_be_u16(o, self.hashed_area.len() as u16)?;
        o.write_all(&self.hashed_area[..])?;

        // XXX: Return an error.
        assert!(self.unhashed_area.len() <= std::u16::MAX as usize);
        write_be_u16(o, self.unhashed_area.len() as u16)?;
        o.write_all(&self.unhashed_area[..])?;

        write_byte(o, self.hash_prefix[0])?;
        write_byte(o, self.hash_prefix[1])?;

        o.write_all(&self.mpis[..])?;

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(4096);
        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o
    }
}

impl Key {
    /// Writes a serialized version of the specified `Key` packet to
    /// `o`.
    pub fn serialize<W: io::Write>(&self, o: &mut W, tag: Tag)
            -> Result<(), io::Error> {
        assert!(tag == Tag::PublicKey
                || tag == Tag::PublicSubkey
                || tag == Tag::SecretKey
                || tag == Tag::SecretSubkey);

        let len = 1 + 4 + 1 + self.mpis.len();

        write_byte(o, ctb_new(tag))?;
        o.write_all(&body_length_new_format(BodyLength::Full(len as u32))[..])?;

        // XXX: Return an error.
        assert_eq!(self.version, 4);
        write_byte(o, self.version)?;
        write_be_u32(o, self.creation_time)?;
        write_byte(o, self.pk_algo)?;
        o.write_all(&self.mpis[..])?;

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self, tag: Tag) -> Vec<u8> {
        let mut o = Vec::with_capacity(4096);
        // Writing to a vec can't fail.
        self.serialize(&mut o, tag).unwrap();
        o
    }
}

impl UserID {
    /// Writes a serialized version of the specified `UserID` packet to
    /// `o`.
    pub fn serialize<W: io::Write>(&self, o: &mut W)
            -> Result<(), io::Error> {
        let len = self.value.len();

        write_byte(o, ctb_old(Tag::UserID, BodyLength::Full(len as u32)))?;
        o.write_all(&body_length_old_format(BodyLength::Full(len as u32))[..])?;
        o.write_all(&self.value[..])?;

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(16 + self.value.len());
        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o
    }
}

impl UserAttribute {
    /// Writes a serialized version of the specified `UserAttribute`
    /// packet to `o`.
    pub fn serialize<W: io::Write>(&self, o: &mut W)
            -> Result<(), io::Error> {
        let len = self.value.len();

        write_byte(o,
            ctb_old(Tag::UserAttribute, BodyLength::Full(len as u32)))?;
        o.write_all(&body_length_old_format(BodyLength::Full(len as u32))[..])?;
        o.write_all(&self.value[..])?;

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(16 + self.value.len());
        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o
    }
}

impl Literal {
    /// Writes a serialized version of the `Literal` data packet to `o`.
    pub fn serialize<W: io::Write>(&self, o: &mut W)
            -> Result<(), io::Error> {
        let body = if let Some(ref body) = self.common.body {
            &body[..]
        } else {
            &b""[..]
        };

        if TRACE {
            let prefix = &body[..cmp::min(body.len(), 20)];
            eprintln!("literal_serialize({}{}, {} bytes)",
                      String::from_utf8_lossy(prefix),
                      if body.len() > 20 { "..." } else { "" },
                      body.len());
        }

        let filename = if let Some(ref filename) = self.filename {
            let len = cmp::min(filename.len(), 255) as u8;
            &filename[..len as usize]
        } else {
            &b""[..]
        };

        let len = 1 + (1 + filename.len()) + 4 + body.len();

        write_byte(o, ctb_old(Tag::Literal, BodyLength::Full(len as u32)))?;

        o.write_all(&body_length_old_format(BodyLength::Full(len as u32))[..])?;

        write_byte(o, self.format)?;
        write_byte(o, filename.len() as u8)?;
        o.write_all(filename)?;
        write_be_u32(o, self.date)?;
        o.write_all(body)?;

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(
            32 + self.common.body.as_ref().map(|b| b.len()).unwrap_or(0)
            + self.filename.as_ref().map(|b| b.len()).unwrap_or(0));

        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o
    }
}

impl CompressedData {
    /// Writes a serialized version of the specified `CompressedData`
    /// packet to `o`.
    ///
    /// This function works recursively: if the `CompressedData` packet
    /// contains any packets, they are also serialized.
    pub fn serialize<W: io::Write>(&self, o: &mut W)
            -> Result<(), io::Error> {
        use flate2::Compression as FlateCompression;
        use flate2::write::{DeflateEncoder, ZlibEncoder};
        use bzip2::Compression as BzCompression;
        use bzip2::write::BzEncoder;

        if TRACE {
            eprintln!("compress_data_serialize(\
                       algo: {}, {:?} children, {:?} bytes)",
                      self.algo,
                      self.common.children.as_ref().map(
                          |cont| cont.children().len()),
                      self.common.body.as_ref().map(|body| body.len()));
        }

        // Packet header.
        write_byte(o, ctb_new(Tag::CompressedData))?;
        let mut o = PartialBodyFilter::new(o);

        // Compressed data header.
        write_byte(&mut o, self.algo)?;

        // Create an appropriate filter.
        let mut o : Box<io::Write> = match self.algo {
            0 => Box::new(o),
            1 => Box::new(DeflateEncoder::new(o, FlateCompression::default()))
                    as Box<io::Write>,
            2 => Box::new(ZlibEncoder::new(o, FlateCompression::default()))
                    as Box<io::Write>,
            3 => Box::new(BzEncoder::new(o, BzCompression::Default))
                    as Box<io::Write>,
            _ => unimplemented!(),
        };

        // Serialize the packets.
        if let Some(ref children) = self.common.children {
            for p in children.children() {
                p.serialize(&mut o)?;
            }
        }

        // Append the data.
        if let Some(ref data) = self.common.body {
            o.write_all(data)?;
        }

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(4 * 1024 * 1024);

        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o.shrink_to_fit();
        o
    }
}

impl Packet {
    /// Writes a serialized version of the specified `Packet` to `o`.
    ///
    /// This function works recursively: if the packet contains any
    /// packets, they are also serialized.
    fn serialize<W: io::Write>(&self, o: &mut W)
            -> Result<(), io::Error> {
        let tag = self.tag();
        match self {
            &Packet::Unknown(ref p) => p.serialize(o),
            &Packet::Signature(ref p) => p.serialize(o),
            &Packet::PublicKey(ref p) => p.serialize(o, tag),
            &Packet::PublicSubkey(ref p) => p.serialize(o, tag),
            &Packet::SecretKey(ref p) => p.serialize(o, tag),
            &Packet::SecretSubkey(ref p) => p.serialize(o, tag),
            &Packet::UserID(ref p) => p.serialize(o),
            &Packet::UserAttribute(ref p) => p.serialize(o),
            &Packet::Literal(ref p) => p.serialize(o),
            &Packet::CompressedData(ref p) => p.serialize(o),
        }
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(4 * 1024 * 1024);

        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o.shrink_to_fit();
        o
    }
}

impl Message {
    /// Writes a serialized version of the specified `Message` to `o`.
    pub fn serialize<W: io::Write>(&self, o: &mut W) -> Result<(), io::Error> {
        for p in self.children() {
            p.serialize(o)?;
        }

        Ok(())
    }

    /// Serializes the packet to a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(4 * 1024 * 1024);

        // Writing to a vec can't fail.
        self.serialize(&mut o).unwrap();
        o.shrink_to_fit();
        o
    }
}

#[cfg(test)]
mod serialize_test {
    use std::fs::File;
    use std::io::Read;

    use super::*;
    use parse::to_unknown_packet;
    use parse::PacketParserBuilder;

    // A convenient function to dump binary data to stdout.
    fn binary_pp(data: &[u8]) -> String {
        let mut output = Vec::with_capacity(data.len() * 2 + 3 * data.len() / 4);

        for i in 0..data.len() {
            if i > 0 && i % (4 * 4 * 2) == 0 {
                output.push('\n' as u8);
            } else {
                if i > 0 && i % 2 == 0 {
                    output.push(' ' as u8);
                }
                if i > 0 && i % (4 * 2) == 0 {
                    output.push(' ' as u8);
                }
            }

            let top = data[i] >> 4;
            let bottom = data[i] & 0xFu8;

            if top < 10u8 {
                output.push('0' as u8 + top)
            } else {
                output.push('A' as u8 + (top - 10u8))
            }

            if bottom < 10u8 {
                output.push('0' as u8 + bottom)
            } else {
                output.push('A' as u8 + (bottom - 10u8))
            }
        }

        // We know the content is valid UTF-8.
        String::from_utf8(output).unwrap()
    }

    // Does a bit-wise comparison of two packets ignoring the CTB
    // format, the body length encoding, and whether partial body
    // length encoding was used.
    fn packets_bitwise_compare(filename: &str, expected: &[u8], got: &[u8]) {
        let expected = to_unknown_packet(expected).unwrap();
        let got = to_unknown_packet(got).unwrap();

        let expected_body = if let Some(ref data) = expected.common.body {
            &data[..]
        } else {
            &b""[..]
        };
        let got_body = if let Some(ref data) = got.common.body {
            &data[..]
        } else {
            &b""[..]
        };

        let mut fail = false;
        if expected.tag != got.tag {
            eprintln!("Expected a {:?}, got a {:?}", expected.tag, got.tag);
            fail = true;
        }
        if expected_body != got_body {
            eprintln!("Packet contents don't match (for {}):",
                      filename);
            eprintln!("Expected ({} bytes):\n{}",
                      expected_body.len(), binary_pp(&expected_body));
            eprintln!("Got ({} bytes):\n{}",
                      got_body.len(), binary_pp(&got_body));
            fail = true;
        }
        if fail {
            panic!("Packets don't match (for {}).", filename);
        }
    }

    #[test]
    fn serialize_test_1() {
        // Given a packet in serialized form:
        //
        // - Parse and reserialize it;
        //
        // - Do a bitwise comparison (modulo the body length encoding)
        //   of the original and reserialized data.
        //
        // Note: This test only works on messages with a single packet.
        //
        // Note: This test does not work with non-deterministic
        // packets, like compressed data packets, since the serialized
        // forms may be different.

        let filenames = [
            "literal-mode-b.gpg",
            "literal-mode-t-partial-body.gpg",

            "sig.gpg",

            "public-key-bare.gpg",
            "public-subkey-bare.gpg",
            "userid-bare.gpg",
        ];

        for filename in filenames.iter() {
            // 1. Read the message byte stream into a local buffer.
            let path = path_to(filename);
            let mut data = Vec::new();
            File::open(&path).expect(&path.to_string_lossy())
                .read_to_end(&mut data).expect("Reading test data");

            // 2. Parse the message.
            let m = Message::from_bytes(&data[..]).unwrap();

            // The following test only works if the message has a
            // single top-level packet.
            assert_eq!(m.children().len(), 1);

            // 3. Serialize the packet it into a local buffer.
            let po = m.descendants().next();
            let mut buffer = Vec::new();
            match po.unwrap() {
                &Packet::Literal(ref l) => {
                    l.serialize(&mut buffer).unwrap();
                },
                &Packet::Signature(ref s) => {
                    s.serialize(&mut buffer).unwrap();
                },
                &Packet::PublicKey(ref pk) => {
                    pk.serialize(&mut buffer, Tag::PublicKey).unwrap();
                },
                &Packet::PublicSubkey(ref pk) => {
                    pk.serialize(&mut buffer, Tag::PublicSubkey).unwrap();
                },
                &Packet::UserID(ref userid) => {
                    userid.serialize(&mut buffer).unwrap();
                },
                ref p => {
                    panic!("Didn't expect a {:?} packet.", p.tag());
                },
            }

            // 4. Modulo the body length encoding, check that the
            // reserialized content is identical to the original data.
            packets_bitwise_compare(filename, &data[..], &buffer[..]);
        }
    }

    #[test]
    fn serialize_test_1_unknown() {
        // This is an variant of serialize_test_1 that tests the
        // unknown packet serializer.
        let filenames = [
            "compressed-data-algo-1.gpg",
            "compressed-data-algo-2.gpg",
            "compressed-data-algo-3.gpg",
            "recursive-2.gpg",
            "recursive-3.gpg",
        ];

        for filename in filenames.iter() {
            // 1. Read the message byte stream into a local buffer.
            let path = path_to(filename);
            let mut data = Vec::new();
            File::open(&path).expect(&path.to_string_lossy())
                .read_to_end(&mut data).expect("Reading test data");

            // 2. Parse the message.
            let u = to_unknown_packet(&data[..]).unwrap();

            // 3. Serialize the packet it into a local buffer.
            let data2 = u.to_vec();

            // 4. Modulo the body length encoding, check that the
            // reserialized content is identical to the original data.
            packets_bitwise_compare(filename, &data[..], &data2[..]);
        }

    }

    #[test]
    fn serialize_test_2() {
        // Given a packet in serialized form:
        //
        // - Parse, reserialize, and reparse it;
        //
        // - Compare the messages.
        //
        // Note: This test only works on messages with a single packet
        // top-level packet.
        //
        // Note: serialize_test_1 is a better test, because it
        // compares the serialized data, but serialize_test_1 doesn't
        // work if the content is non-deterministic.
        let filenames = [
            "compressed-data-algo-1.gpg",
            "compressed-data-algo-2.gpg",
            "compressed-data-algo-3.gpg",
            "recursive-2.gpg",
            "recursive-3.gpg",
        ];

        for filename in filenames.iter() {
            // 1. Read the message into a local buffer.
            let path = path_to(filename);
            let mut data = Vec::new();
            File::open(&path).expect(&path.to_string_lossy())
                .read_to_end(&mut data).expect("Reading test data");

            // 2. Do a shallow parse of the messsage.  In other words,
            // never recurse so that the resulting message only
            // contains the top-level packets.  Any containers will
            // have their raw content stored in packet.content.
            let m = PacketParserBuilder::from_bytes(&data[..]).unwrap()
                .max_recursion_depth(0)
                .buffer_unread_content()
                //.trace()
                .to_message().unwrap();

            // 3. Get the first packet.
            let po = m.descendants().next();
            if let Some(&Packet::CompressedData(ref cd)) = po {
                // 4. Serialize the container.
                let buffer = cd.to_vec();

                // 5. Reparse it.
                let m2 = PacketParserBuilder::from_bytes(&buffer[..]).unwrap()
                    .max_recursion_depth(0)
                    .buffer_unread_content()
                    //.trace()
                    .to_message().unwrap();

                // 6. Make sure the original message matches the
                // serialized and reparsed message.
                if m != m2 {
                    eprintln!("Orig:");
                    let p = m.children().next().unwrap();
                    eprintln!("{:?}", p);
                    let body = &p.body.as_ref().unwrap()[..];
                    eprintln!("Body: {}", body.len());
                    eprintln!("{}", binary_pp(body));

                    eprintln!("Reparsed:");
                    let p = m2.children().next().unwrap();
                    eprintln!("{:?}", p);
                    let body = &p.body.as_ref().unwrap()[..];
                    eprintln!("Body: {}", body.len());
                    eprintln!("{}", binary_pp(body));

                    assert_eq!(m, m2);
                }
            } else {
                panic!("Expected a compressed data data packet.");
            }
        }
    }

    // Create some crazy nesting structures, serialize the messages,
    // reparse them, and make sure we get the same result.
    #[test]
    fn serialize_test_3() {
        // serialize_test_1 and serialize_test_2 parse a byte stream.
        // This tests creates the message, and then serializes and
        // reparses it.

        let mut messages = Vec::new();

        // 1: CompressedData(CompressedData { algo: 0 })
        //  1: Literal(Literal { body: "one (3 bytes)" })
        //  2: Literal(Literal { body: "two (3 bytes)" })
        // 2: Literal(Literal { body: "three (5 bytes)" })
        let mut top_level = Vec::new();
        top_level.push(
            CompressedData::new(0)
                .push(Literal::new('t').body(b"one".to_vec()).to_packet())
                .push(Literal::new('t').body(b"two".to_vec()).to_packet())
                .to_packet());
        top_level.push(Literal::new('t').body(b"three".to_vec()).to_packet());
        messages.push(top_level);

        // 1: CompressedData(CompressedData { algo: 0 })
        //  1: CompressedData(CompressedData { algo: 0 })
        //   1: Literal(Literal { body: "one (3 bytes)" })
        //   2: Literal(Literal { body: "two (3 bytes)" })
        //  2: CompressedData(CompressedData { algo: 0 })
        //   1: Literal(Literal { body: "three (5 bytes)" })
        //   2: Literal(Literal { body: "four (4 bytes)" })
        let mut top_level = Vec::new();
        top_level.push(
            CompressedData::new(0)
                .push(CompressedData::new(0)
                      .push(Literal::new('t').body(b"one".to_vec()).to_packet())
                      .push(Literal::new('t').body(b"two".to_vec()).to_packet())
                      .to_packet())
                .push(CompressedData::new(0)
                      .push(Literal::new('t').body(b"three".to_vec()).to_packet())
                      .push(Literal::new('t').body(b"four".to_vec()).to_packet())
                      .to_packet())
                .to_packet());
        messages.push(top_level);

        // 1: CompressedData(CompressedData { algo: 0 })
        //  1: CompressedData(CompressedData { algo: 0 })
        //   1: CompressedData(CompressedData { algo: 0 })
        //    1: CompressedData(CompressedData { algo: 0 })
        //     1: Literal(Literal { body: "one (3 bytes)" })
        //     2: Literal(Literal { body: "two (3 bytes)" })
        //  2: CompressedData(CompressedData { algo: 0 })
        //   1: CompressedData(CompressedData { algo: 0 })
        //    1: Literal(Literal { body: "three (5 bytes)" })
        //   2: Literal(Literal { body: "four (4 bytes)" })
        let mut top_level = Vec::new();
        top_level.push(
            CompressedData::new(0)
                .push(CompressedData::new(0)
                    .push(CompressedData::new(0)
                        .push(CompressedData::new(0)
                            .push(Literal::new('t').body(b"one".to_vec()).to_packet())
                            .push(Literal::new('t').body(b"two".to_vec()).to_packet())
                            .to_packet())
                        .to_packet())
                    .to_packet())
                .push(CompressedData::new(0)
                    .push(CompressedData::new(0)
                        .push(Literal::new('t').body(b"three".to_vec()).to_packet())
                        .to_packet())
                    .push(Literal::new('t').body(b"four".to_vec()).to_packet())
                    .to_packet())
                .to_packet());
        messages.push(top_level);

        // 1: CompressedData(CompressedData { algo: 0 })
        //  1: Literal(Literal { body: "one (3 bytes)" })
        //  2: Literal(Literal { body: "two (3 bytes)" })
        // 2: Literal(Literal { body: "three (5 bytes)" })
        // 3: Literal(Literal { body: "four (4 bytes)" })
        // 4: CompressedData(CompressedData { algo: 0 })
        //  1: Literal(Literal { body: "five (4 bytes)" })
        //  2: Literal(Literal { body: "six (3 bytes)" })
        let mut top_level = Vec::new();
        top_level.push(
            CompressedData::new(0)
                .push(Literal::new('t').body(b"one".to_vec()).to_packet())
                .push(Literal::new('t').body(b"two".to_vec()).to_packet())
                .to_packet());
        top_level.push(
            Literal::new('t').body(b"three".to_vec()).to_packet());
        top_level.push(
            Literal::new('t').body(b"four".to_vec()).to_packet());
        top_level.push(
            CompressedData::new(0)
                .push(Literal::new('t').body(b"five".to_vec()).to_packet())
                .push(Literal::new('t').body(b"six".to_vec()).to_packet())
                .to_packet());
        messages.push(top_level);

        // 1: UserID(UserID { value: "Foo" })
        let mut top_level = Vec::new();
        top_level.push(UserID::new().userid("Foo").to_packet());
        messages.push(top_level);

        for m in messages.into_iter() {
            // 1. The message.
            let m = Message::from_packets(m);

            m.pretty_print();

            // 2. Serialize the message into a buffer.
            let mut buffer = Vec::new();
            m.clone().serialize(&mut buffer).unwrap();

            // 3. Reparse it.
            let m2 = PacketParserBuilder::from_bytes(&buffer[..]).unwrap()
                //.trace()
                .buffer_unread_content()
                .to_message().unwrap();

            // 4. Compare the messages.
            if m != m2 {
                eprintln!("ORIG...");
                m.pretty_print();
                eprintln!("REPARSED...");
                m2.pretty_print();
                panic!("Reparsed packet does not match original packet!");
            }
        }
    }
}

