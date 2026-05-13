use bigdecimal::{
    BigDecimal, One,
    num_bigint::{BigInt, ToBigInt},
};
use eyre::{ContextCompat, ensure};
use num_bigint::RandBigInt;

use serde::{Deserialize, Serialize};

use crate::{
    data::value::{DataValue, DataValueItem, DataValueNumber, DataValueRef},
    fake::{FakeDataProducer, FakeDataProducerFactory},
    prompt_utils::prompt_decimal,
};

pub struct NumberProducerFactory;

impl FakeDataProducerFactory for NumberProducerFactory {
    fn name(&self) -> String {
        "Number".to_owned()
    }

    fn is_allowed_for(&self, item: &DataValueItem) -> bool {
        item.is_any_number_or_null()
    }

    fn prompt(
        &self,
        _item: &DataValueItem,
    ) -> eyre::Result<Option<Box<dyn super::FakeDataProducer>>> {
        let range = match NumberRange::prompt()? {
            Some(value) => value,
            None => return Ok(None),
        };

        Ok(Some(Box::new(NumberProducer { range })))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberProducer {
    range: NumberRange,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NumberRange {
    Integer {
        min: BigInt,
        max: BigInt,
    },

    Decimal {
        min: BigDecimal,
        max: BigDecimal,
        scale: i64,
    },
}

impl NumberRange {
    pub fn prompt() -> eyre::Result<Option<NumberRange>> {
        let (min, min_decimal) = match prompt_decimal("Enter minimum value", None, None)? {
            Some(value) => value,
            None => return Ok(None),
        };

        let (max, max_decimal) =
            match prompt_decimal("Enter maximum value", Some(min.clone()), None)? {
                Some(value) => value,
                None => return Ok(None),
            };

        let range = if min_decimal || max_decimal {
            let scale = min
                .fractional_digit_count()
                .max(max.fractional_digit_count());

            NumberRange::Decimal { min, max, scale }
        } else {
            let min = min.to_bigint().context("failed to convert min to bigint")?;
            let max = max.to_bigint().context("failed to convert max to bigint")?;

            NumberRange::Integer { min, max }
        };

        Ok(Some(range))
    }

    pub fn fake(&self) -> eyre::Result<DataValueNumber> {
        let mut rng = rand08::thread_rng();
        match self {
            NumberRange::Integer { min, max } => {
                ensure!(min <= max, "min value must not be greater than max");
                let value = random_bigint_between(&mut rng, min, max);
                Ok(DataValueNumber::new(value.to_string()))
            }
            NumberRange::Decimal { min, max, scale } => {
                let value = random_decimal_between(&mut rng, min, max, *scale)?;
                Ok(DataValueNumber::new(value.to_string()))
            }
        }
    }
}

#[typetag::serde(name = "number")]
impl FakeDataProducer for NumberProducer {
    fn produce_fake(&self, _original_value: DataValueRef<'_>) -> eyre::Result<DataValue> {
        let value = self.range.fake()?;
        Ok(DataValue::Number(value))
    }
}

#[inline]
pub fn random_bigint_between(rng: &mut impl RandBigInt, min: &BigInt, max: &BigInt) -> BigInt {
    rng.gen_bigint_range(min, &(max.clone() + BigInt::one()))
}

pub fn random_decimal_between(
    rng: &mut impl RandBigInt,
    min: &BigDecimal,
    max: &BigDecimal,
    scale: i64,
) -> eyre::Result<BigDecimal> {
    ensure!(min <= max, "min value must not be greater than max");

    let scale_factor = BigInt::from(10u32).pow(scale as u32);

    let min_scaled = (min * BigDecimal::from(scale_factor.clone()))
        .to_bigint()
        .context("failed to convert min to scaled integer")?;

    let max_scaled = (max * BigDecimal::from(scale_factor))
        .to_bigint()
        .context("failed to convert max to scaled integer")?
        + BigInt::one();

    let random_scaled = rng.gen_bigint_range(&min_scaled, &max_scaled);

    Ok(BigDecimal::new(random_scaled, scale))
}
