// de.rs
//
// Copyright (c) 2019  Douglas Lau
//
use crate::error::{Error, Result};
use crate::intparse::{self, Integer};
use crate::lines::{DefIter, Define, LineIter};
use serde::de::{
    self, Deserialize, DeserializeSeed, MapAccess, SeqAccess, Visitor,
};
use std::iter::Peekable;

/// Iterator for key/value mappings
struct MappingIter<'a> {
    defs: Peekable<DefIter<'a>>,
}

impl<'a> MappingIter<'a> {
    /// Create a new key/value mapping iterator
    fn new(iter: LineIter<'a>) -> Self {
        let defs = DefIter::new(iter).peekable();
        MappingIter { defs }
    }

    /// Check if the key is valid
    fn check_key(&mut self) -> Result<bool> {
        match self.defs.peek() {
            Some(Define::Invalid(e, ln)) => {
                Err(Error::FailedParse(format!("{:?} {}", e, ln)))
            }
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Peek the current key
    fn peek_key(&mut self) -> Result<&'a str> {
        match self.defs.peek() {
            Some(Define::Invalid(e, ln)) => {
                Err(Error::FailedParse(format!("{:?} {}", e, ln)))
            }
            Some(Define::Valid(_, _, k, _)) => Ok(k),
            None => Err(Error::Eof),
        }
    }

    /// Get the current value
    fn get_value(&mut self) -> Result<&'a str> {
        match self.defs.next() {
            Some(Define::Invalid(e, ln)) => {
                Err(Error::FailedParse(format!("{:?} {}", e, ln)))
            }
            Some(Define::Valid(_, _, _, v)) => Ok(v),
            None => Err(Error::Eof),
        }
    }
}

/// MuON deserializer
pub struct Deserializer<'de> {
    mappings: MappingIter<'de>,
}

impl<'de> Deserializer<'de> {
    fn from_str(input: &'de str) -> Self {
        let mappings = MappingIter::new(LineIter::new(input));
        Deserializer { mappings }
    }
}

/// Create a MuON deserializer from a string slice
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

impl<'de> Deserializer<'de> {
    /// Check if the key is valid
    fn check_key(&mut self) -> Result<bool> {
        self.mappings.check_key()
    }

    /// Peek the current key
    fn peek_key(&mut self) -> Result<&'de str> {
        self.mappings.peek_key()
    }

    /// Get the current value
    fn get_value(&mut self) -> Result<&'de str> {
        self.mappings.get_value()
    }

    fn parse_text(&mut self) -> Result<&'de str> {
        // FIXME: in a list, get one value only
        Ok(self.get_value()?)
    }

    fn parse_char(&mut self) -> Result<char> {
        let text = self.parse_text()?;
        if text.len() == 1 {
            if let Some(c) = text.chars().next() {
                return Ok(c);
            }
        }
        Err(Error::ExpectedChar)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        let value = self.get_value()?;
        match value {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(Error::ExpectedBoolean),
        }
    }

    fn parse_int<T: Integer>(&mut self) -> Result<T> {
        let value = self.get_value()?;
        if let Some(v) = intparse::from_str(value) {
            Ok(v)
        } else {
            Err(Error::ExpectedInteger)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // FIXME: use schema to know what types to return
        unimplemented!();
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // i8 does not impl From<u8>, so use this as workaround
        let v: i16 = self.parse_int()?;
        visitor.visit_i8(v as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_char(self.parse_char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // FIXME: if next line is an "append", build a temp String
        visitor.visit_borrowed_str(self.parse_text()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // FIXME
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::ExpectedEnum)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.peek_key()?)
    }
}

impl<'de> SeqAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // FIXME: check for more at this indent level
        if self.check_key()? {
            seed.deserialize(&mut *self).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'de> MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // FIXME: check for more at this indent level
        if self.check_key()? {
            seed.deserialize(&mut *self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self)
    }
}

#[cfg(test)]
mod test {
    use super::{from_str, Error};
    use serde_derive::Deserialize;

    #[derive(Deserialize, PartialEq, Debug)]
    struct A {
        b: bool,
        uint: u32,
        int: i32,
    }

    #[test]
    fn struct_a() -> Result<(), Box<Error>> {
        let a = "b: false\nuint: 7\nint: -5\n";
        let expected = A {
            b: false,
            uint: 7,
            int: -5,
        };
        assert_eq!(expected, from_str(a)?);
        Ok(())
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct B {
        flags: Vec<bool>,
        values: Vec<String>,
    }

    #[test]
    fn struct_b() -> Result<(), Box<Error>> {
        let b = "flags: false true true false\nvalues: Hello World\n";
        let expected = B {
            flags: vec![false, true, true, false],
            values: vec!["Hello".to_string(), "World".to_string()],
        };
        assert_eq!(expected, from_str(b)?);
        Ok(())
    }
}
