use ethereum_types::{H160, H256, U256};
use itertools::Itertools;
use num::BigUint;
use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use sha2::{Digest, Sha256};

use crate::witness::errors::ProgramError;

/// Construct an integer from its constituent bits (in little-endian order)
pub(crate) fn limb_from_bits_le<P: PackedField>(iter: impl IntoIterator<Item = P>) -> P {
    // TODO: This is technically wrong, as 1 << i won't be canonical for all
    // fields...
    iter.into_iter()
        .enumerate()
        .map(|(i, bit)| bit * P::Scalar::from_canonical_u64(1 << i))
        .sum()
}

/// Construct an integer from its constituent bits (in little-endian order):
/// recursive edition
pub(crate) fn limb_from_bits_le_recursive<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
    iter: impl IntoIterator<Item = ExtensionTarget<D>>,
) -> ExtensionTarget<D> {
    iter.into_iter()
        .enumerate()
        .fold(builder.zero_extension(), |acc, (i, bit)| {
            // TODO: This is technically wrong, as 1 << i won't be canonical for all
            // fields...
            builder.mul_const_add_extension(F::from_canonical_u64(1 << i), bit, acc)
        })
}

/// Returns the lowest LE 32-bit limb of a `U256` as a field element,
/// and errors if the integer is actually greater.
pub(crate) fn u256_to_u32<F: Field>(u256: U256) -> Result<F, ProgramError> {
    if TryInto::<u32>::try_into(u256).is_err() {
        return Err(ProgramError::IntegerTooLarge);
    }

    Ok(F::from_canonical_u32(u256.low_u32()))
}

/// Returns the lowest LE 64-bit word of a `U256` as two field elements
/// each storing a 32-bit limb, and errors if the integer is actually greater.
pub(crate) fn u256_to_u64<F: Field>(u256: U256) -> Result<(F, F), ProgramError> {
    if TryInto::<u64>::try_into(u256).is_err() {
        return Err(ProgramError::IntegerTooLarge);
    }

    Ok((
        F::from_canonical_u32(u256.low_u64() as u32),
        F::from_canonical_u32((u256.low_u64() >> 32) as u32),
    ))
}

/// Safe alternative to `U256::as_usize()`, which errors in case of overflow
/// instead of panicking.
pub(crate) fn u256_to_usize(u256: U256) -> Result<usize, ProgramError> {
    u256.try_into().map_err(|_| ProgramError::IntegerTooLarge)
}

/// Converts a `U256` to a `u8`, erroring in case of overflow instead of
/// panicking.
pub(crate) fn u256_to_u8(u256: U256) -> Result<u8, ProgramError> {
    u256.try_into().map_err(|_| ProgramError::IntegerTooLarge)
}

/// Converts a `U256` to a `bool`, erroring in case of overflow instead of
/// panicking.
pub(crate) fn u256_to_bool(u256: U256) -> Result<bool, ProgramError> {
    if u256 == U256::zero() {
        Ok(false)
    } else if u256 == U256::one() {
        Ok(true)
    } else {
        Err(ProgramError::IntegerTooLarge)
    }
}

/// Converts a `U256` to a `H160`, erroring in case of overflow instead of
/// panicking.
pub(crate) fn u256_to_h160(u256: U256) -> Result<H160, ProgramError> {
    if u256.bits() / 8 > 20 {
        return Err(ProgramError::IntegerTooLarge);
    }
    let mut bytes = [0u8; 32];
    u256.to_big_endian(&mut bytes);
    Ok(H160(
        bytes[12..]
            .try_into()
            .expect("This conversion cannot fail."),
    ))
}

/// Returns the 32-bit little-endian limbs of a `U256`.
pub(crate) fn u256_limbs<F: Field>(u256: U256) -> [F; 8] {
    u256.0
        .into_iter()
        .flat_map(|limb_64| {
            let lo = limb_64 as u32;
            let hi = (limb_64 >> 32) as u32;
            [lo, hi]
        })
        .map(F::from_canonical_u32)
        .collect_vec()
        .try_into()
        .unwrap()
}

/// Returns the 32-bit little-endian limbs of a `H256`.
pub(crate) fn h256_limbs<F: Field>(h256: H256) -> [F; 8] {
    let mut temp_h256 = h256.0;
    temp_h256.reverse();
    temp_h256
        .chunks(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .map(F::from_canonical_u32)
        .collect_vec()
        .try_into()
        .unwrap()
}

pub(crate) const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut indices_arr = [0; N];
    let mut i = 0;
    while i < N {
        indices_arr[i] = i;
        i += 1;
    }
    indices_arr
}

pub(crate) fn addmod(x: U256, y: U256, m: U256) -> U256 {
    if m.is_zero() {
        return m;
    }
    let x = u256_to_biguint(x);
    let y = u256_to_biguint(y);
    let m = u256_to_biguint(m);
    biguint_to_u256((x + y) % m)
}

pub(crate) fn mulmod(x: U256, y: U256, m: U256) -> U256 {
    if m.is_zero() {
        return m;
    }
    let x = u256_to_biguint(x);
    let y = u256_to_biguint(y);
    let m = u256_to_biguint(m);
    biguint_to_u256(x * y % m)
}

pub(crate) fn submod(x: U256, y: U256, m: U256) -> U256 {
    if m.is_zero() {
        return m;
    }
    let mut x = u256_to_biguint(x);
    let y = u256_to_biguint(y);
    let m = u256_to_biguint(m);
    while x < y {
        x += &m;
    }
    biguint_to_u256((x - y) % m)
}

pub(crate) fn u256_to_biguint(x: U256) -> BigUint {
    let mut bytes = [0u8; 32];
    x.to_little_endian(&mut bytes);
    BigUint::from_bytes_le(&bytes)
}

pub(crate) fn biguint_to_u256(x: BigUint) -> U256 {
    let bytes = x.to_bytes_le();
    // This could panic if `bytes.len() > 32` but this is only
    // used here with `BigUint` constructed from `U256`.
    U256::from_little_endian(&bytes)
}

pub(crate) fn mem_vec_to_biguint(x: &[U256]) -> BigUint {
    BigUint::from_slice(
        &x.iter()
            .map(|&n| n.try_into().unwrap())
            .flat_map(|a: u128| {
                [
                    (a % (1 << 32)) as u32,
                    ((a >> 32) % (1 << 32)) as u32,
                    ((a >> 64) % (1 << 32)) as u32,
                    ((a >> 96) % (1 << 32)) as u32,
                ]
            })
            .collect::<Vec<u32>>(),
    )
}

pub(crate) fn biguint_to_mem_vec(x: BigUint) -> Vec<U256> {
    let num_limbs = ((x.bits() + 127) / 128) as usize;

    let mut digits = x.iter_u64_digits();

    let mut mem_vec = Vec::with_capacity(num_limbs);
    while let Some(lo) = digits.next() {
        let hi = digits.next().unwrap_or(0);
        mem_vec.push(U256::from(lo as u128 | (hi as u128) << 64));
    }
    mem_vec
}

pub(crate) fn h2u(h: H256) -> U256 {
    U256::from_big_endian(&h.0)
}

pub(crate) fn get_h160<F: RichField>(slice: &[F]) -> H160 {
    H160::from_slice(
        &slice
            .iter()
            .rev()
            .map(|x| x.to_canonical_u64() as u32)
            .flat_map(|limb| limb.to_be_bytes())
            .collect_vec(),
    )
}

pub(crate) fn get_h256<F: RichField>(slice: &[F]) -> H256 {
    H256::from_slice(
        &slice
            .iter()
            .rev()
            .map(|x| x.to_canonical_u64() as u32)
            .flat_map(|limb| limb.to_be_bytes())
            .collect_vec(),
    )
}

/// Standard Sha2 implementation.
pub(crate) fn sha2(input: Vec<u8>) -> U256 {
    let mut hasher = Sha256::new();
    hasher.update(input);
    U256::from(&hasher.finalize()[..])
}
