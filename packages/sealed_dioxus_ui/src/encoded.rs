use base64::{Engine, engine::general_purpose};
use dioxus::CapturedError;
use lupabase::prelude::*;
use std::{convert::Infallible, marker::PhantomData, ops::Deref, str::FromStr};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Encoded<T>(pub String, pub PhantomData<T>);

impl<T: DatabaseRecord> Encoded<T> {
    pub fn try_as_encoded_bytes(value: T) -> Result<Self, CapturedError> {
        let bytes = CborSerde::try_serialize_as_bytes(value)?;

        return Ok(Self(general_purpose::URL_SAFE.encode(bytes), PhantomData));
    }

    pub fn try_as_decoded_bytes(&self) -> Result<T, CapturedError> {
        let bytes = general_purpose::URL_SAFE.decode(&self.0)?;

        return Ok(CborSerde::try_deserialize_from_bytes(&bytes)?);
    }
}

impl<T> Deref for Encoded<T> {
    type Target = String;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> FromStr for Encoded<T> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self(s.to_string(), PhantomData)) }
}
