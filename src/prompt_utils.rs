use std::{fmt::Display, str::FromStr};

use inquire::{
    Text,
    validator::{ErrorMessage, StringValidator},
};

pub fn prompt_range<E, T>(
    min_message: &str,
    max_message: &str,
    min: T,
    max: T,
) -> eyre::Result<Option<std::ops::Range<T>>>
where
    E: std::error::Error + Send + Sync + 'static,
    T: Display + FromStr<Err = E> + PartialOrd + Ord + Copy + Clone,
{
    let min = match prompt_number(min_message, min, max)? {
        Some(value) => value,
        None => return Ok(None),
    };

    let max = match prompt_number(max_message, min, max)? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(min..max))
}

pub fn prompt_number<E, T>(message: &str, min: T, max: T) -> eyre::Result<Option<T>>
where
    E: std::error::Error + Send + Sync + 'static,
    T: Display + FromStr<Err = E> + PartialOrd + Ord + Copy + Clone,
{
    let validator = RangeNumberValidator { min, max };
    let value = match Text::new(message)
        .with_default(&min.to_string())
        .with_validator(validator)
        .prompt_skippable()?
    {
        Some(value) => value,
        None => return Ok(None),
    };

    let value = value.parse()?;

    Ok(Some(value))
}

#[derive(Clone)]
struct RangeNumberValidator<T> {
    min: T,
    max: T,
}

impl<T> StringValidator for RangeNumberValidator<T>
where
    T: Display + FromStr + PartialOrd + Ord + Clone,
{
    fn validate(
        &self,
        input: &str,
    ) -> Result<inquire::validator::Validation, inquire::CustomUserError> {
        let value: T = match input.parse() {
            Ok(value) => value,
            Err(_) => {
                return Ok(inquire::validator::Validation::Invalid(
                    ErrorMessage::Custom("value is not a valid number".to_owned()),
                ));
            }
        };

        if value < self.min {
            return Ok(inquire::validator::Validation::Invalid(
                ErrorMessage::Custom(format!("value cannot be less than {}", self.min)),
            ));
        }

        if value > self.max {
            return Ok(inquire::validator::Validation::Invalid(
                ErrorMessage::Custom(format!("value cannot be greater than {}", self.max)),
            ));
        }

        Ok(inquire::validator::Validation::Valid)
    }
}
