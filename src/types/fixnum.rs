use super::primitives::{bigint_to_fr, u64_to_fr, Fr};
use num_traits::pow::Pow;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use std::convert::TryInto;

use anyhow::bail;
use anyhow::Result;
use num_bigint::BigInt;

pub fn decimal_to_u64(num: &Decimal, prec: u32) -> u64 {
    let prec_mul = Decimal::new(10, 0).powi(prec as u64);
    let adjusted = num * prec_mul;
    adjusted.floor().to_u64().unwrap()
}

pub fn decimal_to_fr(num: &Decimal, prec: u32) -> Fr {
    // TODO: is u64 enough?
    u64_to_fr(decimal_to_u64(num, prec))
}

pub fn decimal_to_amount(num: &Decimal, prec: u32) -> Float832 {
    Float832::from_decimal(num, prec).unwrap()
}

#[cfg(test)]
#[test]
fn test_decimal_to_fr() {
    let pi = Decimal::new(3141, 3);
    let out = decimal_to_fr(&pi, 3);
    assert_eq!(
        "Fr(0x0000000000000000000000000000000000000000000000000000000000000c45)",
        out.to_string()
    );
}

#[derive(Debug, Clone, Copy)]
pub struct Float832 {
    pub exponent: u8,
    pub significand: u32,
}

impl Float832 {
    pub fn to_bigint(&self) -> BigInt {
        let s = BigInt::from(self.significand);
        s * BigInt::from(10).pow(self.exponent)
    }
    pub fn to_fr(&self) -> Fr {
        bigint_to_fr(self.to_bigint())
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut result = self.exponent.to_be_bytes().to_vec();
        result.append(&mut self.significand.to_be_bytes().to_vec());
        result
    }
    pub fn decode(data: &[u8]) -> Result<Self> {
        let exponent = u8::from_be_bytes(data[0..1].try_into()?);
        let significand = u32::from_be_bytes(data[1..5].try_into()?);
        Ok(Self { exponent, significand })
    }
    pub fn to_decimal(&self, prec: u32) -> Decimal {
        // for example, (significand:1, exponent:17) means 10**17, when prec is 18,
        // it is 0.1 (ETH)
        Decimal::new(self.significand as i64, 0) * Decimal::new(10, 0).powi(self.exponent as u64) / Decimal::new(10, 0).powi(prec as u64)
    }
    pub fn from_decimal(d: &Decimal, prec: u32) -> Result<Self> {
        // if d is "0.1" and prec is 18, result is (significand:1, exponent:17)
        if d.is_zero() {
            return Ok(Self {
                exponent: 0,
                significand: 0,
            });
        }
        let ten = Decimal::new(10, 0);
        let exp = ten.powi(prec as u64);
        let mut n = d * exp;
        if n != n.floor() {
            bail!("decimal precision error");
        }
        let mut exponent = 0;
        loop {
            let next = n / ten;
            if next == next.floor() {
                exponent += 1;
                n = next;
            } else {
                break;
            }
        }
        if n > Decimal::new(std::u32::MAX as i64, 0) {
            bail!("invalid precision {} {}", d, prec);
        }
        // TODO: a better way...
        let significand: u32 = n.floor().to_string().parse::<u32>()?;
        Ok(Float832 { exponent, significand })
    }
}

#[cfg(test)]
#[test]
fn test_float832() {
    use std::str::FromStr;
    // 1.23456 * 10**18
    let d0 = Decimal::new(123456, 5);
    let f = Float832::from_decimal(&d0, 18).unwrap();
    assert_eq!(f.exponent, 13);
    assert_eq!(f.significand, 123456);
    let d = f.to_decimal(18);
    assert_eq!(d, Decimal::from_str("1.23456").unwrap());
    let f2 = Float832::decode(&f.encode()).unwrap();
    assert_eq!(f2.exponent, 13);
    assert_eq!(f2.significand, 123456);
}
