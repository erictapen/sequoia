//! Handles PacketPiles.
//!
//! Wraps [`sequoia-openpgp::PacketPile`].
//!
//! [`sequoia-openpgp::PacketPile`]: ../../../sequoia_openpgp/struct.PacketPile.html

use std::slice;
use std::io::{Read, Write};
use libc::{uint8_t, c_char, size_t};

extern crate sequoia_openpgp;
use self::sequoia_openpgp::{
    PacketPile,
    parse::Parse,
    serialize::Serialize,
};

use ::core::Context;
use ::error::Status;

/// Deserializes the OpenPGP message stored in a `std::io::Read`
/// object.
///
/// Although this method is easier to use to parse an OpenPGP
/// message than a `PacketParser` or a `PacketPileParser`, this
/// interface buffers the whole message in memory.  Thus, the
/// caller must be certain that the *deserialized* message is not
/// too large.
///
/// Note: this interface *does* buffer the contents of packets.
#[::ffi_catch_abort] #[no_mangle]
pub extern "system" fn sq_packet_pile_from_reader(ctx: *mut Context,
                                                  reader: *mut Box<Read>)
                                                  -> *mut PacketPile {
    let ctx = ffi_param_ref_mut!(ctx);
    ffi_make_fry_from_ctx!(ctx);
    let reader = ffi_param_ref_mut!(reader);
    ffi_try_box!(PacketPile::from_reader(reader))
}

/// Deserializes the OpenPGP message stored in the file named by
/// `filename`.
///
/// See `sq_packet_pile_from_reader` for more details and caveats.
#[::ffi_catch_abort] #[no_mangle]
pub extern "system" fn sq_packet_pile_from_file(ctx: *mut Context,
                                                filename: *const c_char)
                                                -> *mut PacketPile {
    let ctx = ffi_param_ref_mut!(ctx);
    ffi_make_fry_from_ctx!(ctx);
    let filename = ffi_param_cstr!(filename).to_string_lossy().into_owned();
    ffi_try_box!(PacketPile::from_file(&filename))
}

/// Deserializes the OpenPGP message stored in the provided buffer.
///
/// See `sq_packet_pile_from_reader` for more details and caveats.
#[::ffi_catch_abort] #[no_mangle]
pub extern "system" fn sq_packet_pile_from_bytes(ctx: *mut Context,
                                                 b: *const uint8_t, len: size_t)
                                                 -> *mut PacketPile {
    let ctx = ffi_param_ref_mut!(ctx);
    ffi_make_fry_from_ctx!(ctx);
    assert!(!b.is_null());
    let buf = unsafe {
        slice::from_raw_parts(b, len as usize)
    };

    ffi_try_box!(PacketPile::from_bytes(buf))
}

/// Frees the packet_pile.
#[::ffi_catch_abort] #[no_mangle]
pub extern "system" fn sq_packet_pile_free(packet_pile: Option<&mut PacketPile>)
{
    ffi_free!(packet_pile)
}

/// Clones the PacketPile.
#[::ffi_catch_abort] #[no_mangle]
pub extern "system" fn sq_packet_pile_clone(packet_pile: *const PacketPile)
                                            -> *mut PacketPile {
    let packet_pile = ffi_param_ref!(packet_pile);
    box_raw!(packet_pile.clone())
}

/// Serializes the packet pile.
#[::ffi_catch_abort] #[no_mangle]
pub extern "system" fn sq_packet_pile_serialize(ctx: *mut Context,
                                                packet_pile: *const PacketPile,
                                                writer: *mut Box<Write>)
                                                -> Status {
    let ctx = ffi_param_ref_mut!(ctx);
    ffi_make_fry_from_ctx!(ctx);
    let packet_pile = ffi_param_ref!(packet_pile);
    let writer = ffi_param_ref_mut!(writer);
    ffi_try_status!(packet_pile.serialize(writer))
}
