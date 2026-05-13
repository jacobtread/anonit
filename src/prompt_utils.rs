use std::{fmt::Display, ops::Add, path::PathBuf, str::FromStr};

use bigdecimal::BigDecimal;
use inquire::{
    Text,
    validator::{ErrorMessage, StringValidator},
};

pub fn prompt_file_path(message: &str) -> eyre::Result<Option<PathBuf>> {
    let value = match Text::new(message)
        .with_validator(FilePathValidator)
        .prompt_skippable()?
    {
        Some(value) => value,
        None => return Ok(None),
    };

    let path: PathBuf = value.parse()?;
    Ok(Some(path))
}

pub fn prompt_range<E, T>(
    min_message: &str,
    max_message: &str,
    min: T,
    max: T,
) -> eyre::Result<Option<std::ops::Range<T>>>
where
    E: std::error::Error + Send + Sync + 'static,
    T: Display
        + FromStr<Err = E>
        + PartialOrd
        + Ord
        + Copy
        + Clone
        + Add<Output = T>
        + From<u8>
        + Ord,
{
    let min = match prompt_number(min_message, min, max)? {
        Some(value) => value,
        None => return Ok(None),
    };

    let max = match prompt_number(max_message, min, max)? {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(inclusive_to_exclusive(min..=max))
}

fn inclusive_to_exclusive<T>(range: std::ops::RangeInclusive<T>) -> Option<std::ops::Range<T>>
where
    T: Copy + Add<Output = T> + From<u8> + Ord,
{
    let end = *range.end();
    let next = end + T::from(1);

    if next > end {
        Some(*range.start()..next)
    } else {
        None
    }
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
struct FilePathValidator;

impl StringValidator for FilePathValidator {
    fn validate(
        &self,
        input: &str,
    ) -> Result<inquire::validator::Validation, inquire::CustomUserError> {
        let path: PathBuf = PathBuf::from_str(input)?;
        if path.is_dir() {
            return Ok(inquire::validator::Validation::Invalid(
                ErrorMessage::Custom("path must be to a file".to_owned()),
            ));
        }

        if !path.is_file() {
            return Ok(inquire::validator::Validation::Invalid(
                ErrorMessage::Custom("file does not exist".to_owned()),
            ));
        }

        Ok(inquire::validator::Validation::Valid)
    }
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

pub fn prompt_decimal(
    message: &str,
    min: Option<BigDecimal>,
    max: Option<BigDecimal>,
) -> eyre::Result<Option<(BigDecimal, bool)>> {
    let default = match &min {
        Some(value) => value.to_string(),
        None => "".to_string(),
    };
    let validator = DecimalNumberValidator { min, max };
    let value = match Text::new(message)
        .with_default(&default)
        .with_validator(validator)
        .prompt_skippable()?
    {
        Some(value) => value,
        None => return Ok(None),
    };

    let has_decimal = value.contains('.');
    let value = value.parse()?;

    Ok(Some((value, has_decimal)))
}

#[derive(Clone)]
struct DecimalNumberValidator {
    min: Option<BigDecimal>,
    max: Option<BigDecimal>,
}

impl StringValidator for DecimalNumberValidator {
    fn validate(
        &self,
        input: &str,
    ) -> Result<inquire::validator::Validation, inquire::CustomUserError> {
        let value: BigDecimal = match input.parse() {
            Ok(value) => value,
            Err(_) => {
                return Ok(inquire::validator::Validation::Invalid(
                    ErrorMessage::Custom("value is not a valid number".to_owned()),
                ));
            }
        };

        if let Some(min) = self.min.as_ref()
            && value.lt(min)
        {
            return Ok(inquire::validator::Validation::Invalid(
                ErrorMessage::Custom(format!("value cannot be less than {}", min)),
            ));
        }

        if let Some(max) = self.max.as_ref()
            && value.gt(max)
        {
            return Ok(inquire::validator::Validation::Invalid(
                ErrorMessage::Custom(format!("value cannot be greater than {}", max)),
            ));
        }

        Ok(inquire::validator::Validation::Valid)
    }
}
