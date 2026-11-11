//! Behind fake: a mock row's values by SQLite type, CHECK lists and UUIDv7.

use std::time::{SystemTime, UNIX_EPOCH};

use fake::rand::{Rng, RngExt};
use fake::{Fake, faker::lorem::en::Word};

use super::{Column, Fill, Type};
use crate::value::Value;

/// One row's values, every column but those left to their default.
pub(super) fn row<R: Rng + ?Sized>(rng: &mut R, fill: &[Column]) -> Vec<(&'static str, Value)> {
    fill.iter()
        .filter_map(|column| {
            let value = match column.fill {
                Fill::Default => return None,
                Fill::Null => Value::Null,
                Fill::Uuid7 => Value::Text(uuid7(rng)),
                Fill::PositiveInteger => Value::Integer(rng.random_range(1..=i64::MAX)),
                Fill::Value { nullable: true, .. } if rng.random_bool(0.5) => Value::Null,
                Fill::Value {
                    list: Some(list), ..
                } if !list.is_empty() => {
                    Value::Text(list[rng.random_range(..list.len())].to_owned())
                }
                Fill::Value { ty, .. } => by_type(rng, ty),
            };
            Some((column.name, value))
        })
        .collect()
}

/// Any value of the type.
fn by_type<R: Rng + ?Sized>(rng: &mut R, ty: Type) -> Value {
    match ty {
        Type::Integer | Type::Numeric => Value::Integer(rng.random()),
        // A word and a random suffix: close to unique, so a UNIQUE column or
        // index takes every row of a mock.
        Type::Text => Value::Text(format!(
            "{}-{:08x}",
            Word().fake_with_rng::<&str, _>(rng),
            rng.random::<u32>()
        )),
        Type::Real => Value::Real(rng.random()),
        Type::Blob => {
            let len = rng.random_range(0..16);
            Value::Blob((0..len).map(|_| rng.random()).collect())
        }
    }
}

/// A UUIDv7 (RFC 9562): 48 bits of Unix milliseconds, version 7, 12 random
/// bits, variant 10 and 62 random bits.
fn uuid7<R: Rng + ?Sized>(rng: &mut R) -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        & 0xffff_ffff_ffff;
    let a: u16 = rng.random::<u16>() & 0x0fff;
    let b: u64 = rng.random::<u64>() & 0x3fff_ffff_ffff_ffff;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        ms >> 16,
        ms & 0xffff,
        0x7000 | a,
        0x8000 | (b >> 48),
        b & 0xffff_ffff_ffff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_uuid7_has_its_version_and_variant() {
        let id = uuid7(&mut fake::rand::rng());
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(
            parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
            [8, 4, 4, 4, 12]
        );
        assert!(parts[2].starts_with('7'));
        assert!(parts[3].starts_with(['8', '9', 'a', 'b']));
    }

    #[test]
    fn text_values_do_not_repeat() {
        let mut rng = fake::rand::rng();
        let values: std::collections::HashSet<String> = (0..1000)
            .map(|_| match by_type(&mut rng, Type::Text) {
                Value::Text(v) => v,
                other => panic!("not text: {other:?}"),
            })
            .collect();
        assert_eq!(values.len(), 1000);
    }

    #[test]
    fn a_default_column_is_left_out_and_a_list_column_takes_a_listed_value() {
        let fill = [
            Column {
                name: "d",
                fill: Fill::Default,
            },
            Column {
                name: "k",
                fill: Fill::Value {
                    ty: Type::Text,
                    nullable: false,
                    list: Some(&["memo", "todo"]),
                },
            },
        ];
        let mut rng = fake::rand::rng();
        for _ in 0..20 {
            let row = row(&mut rng, &fill);
            assert_eq!(row.len(), 1);
            assert!(matches!(&row[0], ("k", Value::Text(v)) if v == "memo" || v == "todo"));
        }
    }
}
