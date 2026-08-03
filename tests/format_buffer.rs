use serde::Deserialize;
use std::collections::BTreeMap;

#[test]
fn internally_tagged_map_with_integer_keys() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "type")]
    enum Enum {
        Map(BTreeMap<u32, ()>),
    }

    assert_eq!(
        serde_json::from_str::<Enum>(r#"{"type":"Map","1":null}"#).unwrap(),
        Enum::Map(BTreeMap::from([(1, ())])),
    );
}

#[test]
fn flattened_map_with_integer_keys() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Struct {
        known: u32,
        #[serde(flatten)]
        extra: BTreeMap<u32, u32>,
    }

    assert_eq!(
        serde_json::from_str::<Struct>(r#"{"known":1,"2":3}"#).unwrap(),
        Struct {
            known: 1,
            extra: BTreeMap::from([(2, 3)]),
        },
    );
}

#[cfg(feature = "arbitrary_precision")]
#[test]
fn untagged_u128_with_arbitrary_precision() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Enum {
        Integer(u128),
    }

    assert_eq!(
        serde_json::from_str::<Enum>("340282366920938463463374607431768211455").unwrap(),
        Enum::Integer(u128::MAX),
    );
}

// https://github.com/serde-rs/serde/issues/1998
#[cfg(feature = "arbitrary_precision")]
#[test]
fn issue_1998_untagged_integer_with_arbitrary_precision() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Enum {
        Integer(i32),
    }

    assert_eq!(
        serde_json::from_str::<Enum>("10").unwrap(),
        Enum::Integer(10),
    );
}

// https://github.com/serde-rs/serde/issues/2623
#[cfg(feature = "arbitrary_precision")]
#[test]
fn issue_2623_internally_tagged_float_with_arbitrary_precision() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "name", rename_all = "lowercase")]
    enum Fruit {
        Apple(Apple),
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Apple {
        size: f64,
    }

    assert_eq!(
        serde_json::from_str::<Fruit>(r#"{"name":"apple","size":1.23}"#).unwrap(),
        Fruit::Apple(Apple { size: 1.23 }),
    );
}

// https://github.com/serde-rs/serde/issues/2661
#[cfg(feature = "arbitrary_precision")]
#[test]
fn issue_2661_untagged_float_with_arbitrary_precision() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Enum {
        Float(f64),
    }

    assert_eq!(
        serde_json::from_str::<Enum>("123.45").unwrap(),
        Enum::Float(123.45),
    );
}

// https://github.com/serde-rs/serde/issues/2748
#[cfg(feature = "arbitrary_precision")]
#[test]
fn issue_2748_flattened_float_with_arbitrary_precision() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Outer {
        #[serde(flatten)]
        inner: Inner,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Inner {
        value: f64,
    }

    assert_eq!(
        serde_json::from_str::<Outer>(r#"{"value":1.0}"#).unwrap(),
        Outer {
            inner: Inner { value: 1.0 },
        },
    );
}

// https://github.com/serde-rs/serde/issues/2903
#[cfg(feature = "arbitrary_precision")]
#[test]
fn issue_2903_adjacently_tagged_trailing_zero_float() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "type", content = "data")]
    enum Data {
        #[serde(alias = "a")]
        A(Info),
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Info {
        timestamp: f64,
    }

    let value = serde_json::from_str(r#"{"type":"a","data":{"timestamp":1.10}}"#).unwrap();
    assert_eq!(
        serde_json::from_value::<Data>(value).unwrap(),
        Data::A(Info { timestamp: 1.1 }),
    );
}

#[cfg(feature = "raw_value")]
#[test]
fn raw_value_inside_untagged_enum() {
    use serde_json::value::RawValue;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Enum {
        Raw(Box<RawValue>),
    }

    let Enum::Raw(raw) = serde_json::from_str::<Enum>(r#"{"key": [1, 2]}"#).unwrap();
    assert_eq!(raw.get(), r#"{"key": [1, 2]}"#);
}

// https://github.com/serde-rs/serde/issues/2213
#[cfg(feature = "raw_value")]
#[test]
fn issue_2213_raw_value_inside_flattened_struct() {
    use serde_json::value::RawValue;

    #[derive(Debug, Deserialize)]
    struct Outer {
        #[serde(flatten)]
        inner: Inner,
    }

    #[derive(Debug, Deserialize)]
    struct Inner {
        raw: Box<RawValue>,
    }

    let outer = serde_json::from_str::<Outer>(r#"{"raw":{}}"#).unwrap();
    assert_eq!(outer.inner.raw.get(), "{}");
}

// https://github.com/serde-rs/serde/issues/1911
#[cfg(feature = "raw_value")]
#[test]
#[allow(dead_code)]
fn issue_1911_borrowed_raw_value_in_multiple_untagged_variants() {
    use serde_json::value::RawValue;

    #[derive(Debug, Deserialize)]
    struct A<'a> {
        id: &'a str,
        #[serde(borrow)]
        a: &'a RawValue,
    }

    #[derive(Debug, Deserialize)]
    struct B<'a> {
        id: &'a str,
        #[serde(borrow)]
        b: &'a RawValue,
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Enum<'a> {
        #[serde(borrow)]
        A(A<'a>),
        #[serde(borrow)]
        B(B<'a>),
    }

    let json = r#"{"id":"second","b":{"preserved": true}}"#;
    let Enum::B(B { id, b }) = serde_json::from_str::<Enum<'_>>(json).unwrap() else {
        panic!("expected second untagged variant");
    };
    assert_eq!(id, "second");
    assert_eq!(b.get(), r#"{"preserved": true}"#);
    assert!(b.get().as_ptr() >= json.as_ptr());
    assert!(b.get().as_ptr() < json[json.len()..].as_ptr());
}

#[cfg(feature = "raw_value")]
#[test]
fn borrowed_raw_value_inside_nested_untagged_enum() {
    use serde_json::value::RawValue;

    #[derive(Debug, Deserialize)]
    struct Pair<'a>(u8, #[serde(borrow)] &'a RawValue);

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Enum<'a> {
        Pair(#[serde(borrow)] Pair<'a>),
    }

    let json = r#"[1, { "key": [1, 2] }]"#;
    let Enum::Pair(Pair(number, raw)) = serde_json::from_str::<Enum<'_>>(json).unwrap();
    assert_eq!(number, 1);
    assert_eq!(raw.get(), r#"{ "key": [1, 2] }"#);
    assert!(raw.get().as_ptr() >= json.as_ptr());
    assert!(raw.get().as_ptr() < json[json.len()..].as_ptr());
}

#[cfg(feature = "raw_value")]
#[test]
fn owned_raw_value_inside_nested_untagged_enum() {
    use serde_json::value::RawValue;

    #[derive(Debug, Deserialize)]
    struct Pair(u8, Box<RawValue>);

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Enum {
        Pair(Pair),
    }

    let json = br#"[1, { "key": [1, 2] }]"#;
    let Enum::Pair(Pair(number, raw)) = serde_json::from_reader::<_, Enum>(&json[..]).unwrap();
    assert_eq!(number, 1);
    assert_eq!(raw.get(), r#"{ "key": [1, 2] }"#);
}

#[cfg(feature = "raw_value")]
#[test]
fn replay_errors_retain_source_position() {
    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    enum Enum {
        Struct {
            #[serde(rename = "value")]
            _value: u32,
        },
    }

    let json = "{\n  \"type\": \"Struct\",\n  \"value\": \"bad\"\n}";
    let error = serde_json::from_str::<Enum>(json).unwrap_err();
    assert_eq!((error.line(), error.column()), (3, 16));
}

#[cfg(any(feature = "arbitrary_precision", feature = "raw_value"))]
mod extension {
    use serde::de::{self, DeserializeExtension, Deserializer};
    #[cfg(feature = "arbitrary_precision")]
    use serde::de::{ExtensionAccess, Visitor};
    use serde::ser::{SerializeExtension, Serializer};
    use serde::{Deserialize, ExtensionId, Serialize};

    const NUMBER: ExtensionId = ExtensionId::new("serde_json/number@1");
    const RAW_VALUE: ExtensionId = ExtensionId::new("serde_json/raw_value@1");

    struct Extended<'a> {
        id: ExtensionId,
        payload: &'a str,
    }

    impl Serialize for Extended<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_extension(self)
        }
    }

    impl SerializeExtension for Extended<'_> {
        fn id(&self) -> ExtensionId {
            self.id
        }

        fn serialize_payload<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.payload)
        }

        fn serialize_fallback<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str("fallback")
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    #[test]
    fn number_extension_is_native() {
        struct NativeNumber(String);

        impl<'de> Deserialize<'de> for NativeNumber {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct Request;

                impl<'de, E> DeserializeExtension<'de, E> for Request
                where
                    E: de::Error,
                {
                    type Value = NativeNumber;

                    fn id(&self) -> ExtensionId {
                        NUMBER
                    }

                    fn deserialize_payload<D>(self, deserializer: D) -> Result<Self::Value, E>
                    where
                        D: Deserializer<'de, Error = E>,
                    {
                        String::deserialize(deserializer).map(NativeNumber)
                    }

                    fn deserialize_fallback<D>(self, _deserializer: D) -> Result<Self::Value, E>
                    where
                        D: Deserializer<'de, Error = E>,
                    {
                        Err(de::Error::custom("number extension was not recognized"))
                    }
                }

                deserializer.deserialize_extension(Request)
            }
        }

        struct VisitedNumber(String);

        impl<'de> Deserialize<'de> for VisitedNumber {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct NumberVisitor;

                impl<'de> Visitor<'de> for NumberVisitor {
                    type Value = VisitedNumber;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("an extended JSON number")
                    }

                    fn visit_extension<A>(self, mut access: A) -> Result<Self::Value, A::Error>
                    where
                        A: ExtensionAccess<'de>,
                    {
                        if access.id() == NUMBER {
                            access.deserialize_payload(self)
                        } else {
                            access.deserialize_fallback(self)
                        }
                    }

                    fn visit_str<E>(self, number: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(VisitedNumber(number.to_owned()))
                    }

                    fn visit_string<E>(self, number: String) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(VisitedNumber(number))
                    }
                }

                deserializer.deserialize_any(NumberVisitor)
            }
        }

        let number = "340282366920938463463374607431768211456";
        assert_eq!(
            serde_json::from_str::<NativeNumber>(number).unwrap().0,
            number
        );
        assert_eq!(
            serde_json::from_str::<VisitedNumber>(number).unwrap().0,
            number
        );
        assert_eq!(
            serde_json::to_string(&Extended {
                id: NUMBER,
                payload: number,
            })
            .unwrap(),
            number,
        );
    }

    #[cfg(feature = "raw_value")]
    #[test]
    fn raw_value_extension_is_native_and_borrowed() {
        struct NativeRaw<'a>(&'a str);

        impl<'de> Deserialize<'de> for NativeRaw<'de> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct Request;

                impl<'de, E> DeserializeExtension<'de, E> for Request
                where
                    E: de::Error,
                {
                    type Value = NativeRaw<'de>;

                    fn id(&self) -> ExtensionId {
                        RAW_VALUE
                    }

                    fn deserialize_payload<D>(self, deserializer: D) -> Result<Self::Value, E>
                    where
                        D: Deserializer<'de, Error = E>,
                    {
                        <&str>::deserialize(deserializer).map(NativeRaw)
                    }

                    fn deserialize_fallback<D>(self, _deserializer: D) -> Result<Self::Value, E>
                    where
                        D: Deserializer<'de, Error = E>,
                    {
                        Err(de::Error::custom("raw value extension was not recognized"))
                    }
                }

                deserializer.deserialize_extension(Request)
            }
        }

        let json = r#"{ "key": [1, 2] }"#;
        let raw = serde_json::from_str::<NativeRaw<'_>>(json).unwrap().0;
        assert_eq!(raw, json);
        assert_eq!(raw.as_ptr(), json.as_ptr());
        assert_eq!(
            serde_json::to_string(&Extended {
                id: RAW_VALUE,
                payload: json,
            })
            .unwrap(),
            json,
        );
    }
}
