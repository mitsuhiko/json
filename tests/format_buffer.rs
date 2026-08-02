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
    assert_eq!(raw.get(), r#"{"key":[1,2]}"#);
}
