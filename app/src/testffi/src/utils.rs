#![allow(unused)]
use anyhow::Context;
use jni::{
    objects::{JIntArray, JString},
    JNIEnv,
};

macro_rules! jtry {
    ($env:expr, $result:expr) => {
        match $result {
            Ok(x) => x,
            Err(e) => {
                log::error!("Got an exception: {e:?}");
                $env.throw_new("java/lang/Exception", format!("Rust Error: {e:?}"))
                    .unwrap();
                return std::ptr::null_mut();
            }
        }
    };
}

pub fn jni_to_string(env: &mut JNIEnv, data: &JString) -> anyhow::Result<String> {
    let data = env.get_string(data).context("Expected a JString!")?;
    let str = data.to_str()?.to_string();
    Ok(str)
}

pub fn jni_to_int_array(env: &mut JNIEnv, data: &JIntArray) -> anyhow::Result<Vec<i32>> {
    let mut buf = vec![0; env.get_array_length(data)? as usize];
    env.get_int_array_region(data, 0, &mut buf).unwrap();
    Ok(buf)
}

pub enum MethodType {
    String,
    I32,
    I64,
    F32,
    F64,
    Char,
    Bool,
    Qualified(String),
    Arr(Box<MethodType>),
}

impl MethodType {
    // https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/types.html#type_signatures
    fn to_jni_type(&self) -> String {
        match self {
            Self::Bool => "Z".to_string(),
            Self::Char => "C".to_string(),
            Self::I32 => "I".to_string(),
            Self::I64 => "L".to_string(),
            Self::F32 => "F".to_string(),
            Self::F64 => "D".to_string(),
            Self::Qualified(name) => format!("L{name};"),
            Self::String => MethodType::Qualified("java/lang/String".to_string()).to_jni_type(),
            Self::Arr(ty) => format!("[{}", ty.to_jni_type()),
        }
    }
}

pub enum ReturnMethodType {
    String,
    I32,
    I64,
    F32,
    F64,
    Char,
    Bool,
    Qualified(String),
    Arr(Box<MethodType>),
    Void,
}

impl ReturnMethodType {
    // https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/types.html#type_signatures
    pub fn to_jni_type(&self) -> String {
        match self {
            Self::Bool => "Z".to_string(),
            Self::Char => "C".to_string(),
            Self::I32 => "I".to_string(),
            Self::I64 => "L".to_string(),
            Self::F32 => "F".to_string(),
            Self::F64 => "D".to_string(),
            Self::Qualified(name) => format!("L{name};"),
            Self::String => MethodType::Qualified("java/lang/String".to_string()).to_jni_type(),
            Self::Arr(ty) => format!("[{}", ty.to_jni_type()),
            Self::Void => "V".to_string(),
        }
    }
}

pub fn get_jni_signature(method_types: &[MethodType], ret: ReturnMethodType) -> String {
    let mut buf = String::new();
    for ty in method_types {
        buf.push_str(&ty.to_jni_type());
    }
    buf = format!("({buf}){}", ret.to_jni_type());
    buf
}
