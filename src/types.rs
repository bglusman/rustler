extern crate ruster_unsafe;
extern crate libc;

use super::{ NifEnv, NifTerm, NifError };

pub trait NifEncoder {
    fn encode<'a>(&self, env: &'a NifEnv) -> NifTerm<'a>;
}
pub trait NifDecoder<'a>: Sized+'a {
    fn decode(term: NifTerm<'a>, env: &NifEnv) -> Result<Self, NifError>;
}

macro_rules! impl_number_transcoder {
    ($dec_type:ty, $nif_type:ty, $encode_fun:ident, $decode_fun:ident) => {
        impl NifEncoder for $dec_type {
            fn encode<'a>(&self, env: &'a NifEnv) -> NifTerm<'a> {
                #![allow(unused_unsafe)]
                NifTerm::new(env, unsafe { ruster_unsafe::$encode_fun(env.env, *self as $nif_type) })
            }
        }
        impl<'a> NifDecoder<'a> for $dec_type {
            fn decode(term: NifTerm, env: &NifEnv) -> Result<$dec_type, NifError> {
                #![allow(unused_unsafe)]
                let mut res: $nif_type = Default::default();
                if unsafe { ruster_unsafe::$decode_fun(env.env, term.term, (&mut res) as *mut $nif_type) } == 0 {
                    return Err(NifError::BadArg);
                }
                Ok(res as $dec_type)
            }
        }
    }
}

// Base number types
impl_number_transcoder!(i32, i32, enif_make_int, enif_get_int);
impl_number_transcoder!(u32, u32, enif_make_uint, enif_get_uint);
impl_number_transcoder!(i64, i64, enif_make_int64, enif_get_int64);
impl_number_transcoder!(u64, u64, enif_make_uint64, enif_get_uint64);
impl_number_transcoder!(f64, f64, enif_make_double, enif_get_double);

// Casted number types
impl_number_transcoder!(i8, i32, enif_make_int, enif_get_int);
impl_number_transcoder!(u8, u32, enif_make_uint, enif_get_uint);
impl_number_transcoder!(i16, i32, enif_make_int, enif_get_int);
impl_number_transcoder!(u16, u32, enif_make_uint, enif_get_uint);
impl_number_transcoder!(f32, f64, enif_make_double, enif_get_double);

use super::atom::{ get_atom };
impl NifEncoder for bool {
    fn encode<'a>(&self, env: &'a NifEnv) -> NifTerm<'a> {
        if *self {
            get_atom("true").unwrap().to_term(env)
        } else {
            get_atom("false").unwrap().to_term(env)
        }
    }
}
impl<'a> NifDecoder<'a> for bool {
    fn decode(term: NifTerm<'a>, env: &NifEnv) -> Result<bool, NifError> {
        Ok(super::atom::is_term_truthy(term, env))
    }
}

impl<'a> NifDecoder<'a> for String {
    fn decode(term: NifTerm<'a>, env: &NifEnv) -> Result<Self, NifError> {
        let string: &str = try!(NifDecoder::decode(term, env));
        Ok(string.to_string())
    }
}
impl<'a> NifDecoder<'a> for &'a str {
    fn decode(term: NifTerm<'a>, env: &NifEnv) -> Result<Self, NifError> {
        let binary = try!(::binary::NifBinary::from_term(term, env));
        match ::std::str::from_utf8(binary.as_slice()) {
            Ok(string) => Ok(string),
            Err(_) => Err(NifError::BadArg),
        }
    }
}

use std::io::Write;

impl NifEncoder for str {
    fn encode<'b>(&self, env: &'b NifEnv) -> NifTerm<'b> {
        let str_len = self.len();
        let mut bin = match ::binary::OwnedNifBinary::alloc(str_len) {
            Some(bin) => bin,
            None => panic!("binary term alloc failed"),
        };
        bin.as_mut_slice().write(self.as_bytes()).expect("memory copy of string failed");
        bin.release(env).get_term(env)
    }
}

impl<'a> NifEncoder for NifTerm<'a> {
    fn encode<'b>(&self, env: &'b NifEnv) -> NifTerm<'b> {
        NifTerm::new(env, self.as_c_arg())
    }
}
impl<'a> NifDecoder<'a> for NifTerm<'a> {
    fn decode(term: NifTerm<'a>, _env: &NifEnv) -> Result<Self, NifError> {
        Ok(term)
    }
}
