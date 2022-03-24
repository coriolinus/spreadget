use serde::{
    de::{Error as _, SeqAccess},
    Deserialize,
};

#[derive(Debug, Clone, Copy)]
pub struct AnonymousLevel {
    pub price: f64,
    pub amount: f64,
}

impl AnonymousLevel {
    pub fn associate(self, exchange: String) -> crate::Level {
        let Self { price, amount } = self;
        crate::Level {
            exchange,
            price,
            amount,
        }
    }
}

impl<'de> Deserialize<'de> for AnonymousLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = AnonymousLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("list of two numbers")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut expect_element = |field_name: &'static str| {
                    seq.next_element::<StringFloat>()?
                        .ok_or_else(|| A::Error::missing_field(field_name))
                };

                let price = expect_element("price")?;
                let amount = expect_element("qty")?;

                if seq.next_element::<StringFloat>()?.is_some() {
                    return Err(A::Error::invalid_length(3, &self));
                }

                Ok(AnonymousLevel {
                    price: price.into(),
                    amount: amount.into(),
                })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

/// This helper type exists so that we can deserialize a string representation of a number into itself.
struct StringFloat(f64);

impl<'de> Deserialize<'de> for StringFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StringFloat;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("string containing a floating point literal")
            }

            /// In case we get a raw float instead of a string, we can handle it.
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(StringFloat(v))
            }

            /// In case we get a string, try to parse it as a float.
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse::<f64>()
                    .map(StringFloat)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl From<StringFloat> for f64 {
    fn from(sf: StringFloat) -> Self {
        sf.0
    }
}
