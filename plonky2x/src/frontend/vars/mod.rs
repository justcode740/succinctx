mod array;
mod boolean;
mod buffer;
mod byte;
mod bytes;
mod bytes32;
mod variable;
use std::fmt::Debug;

pub use array::*;
pub use boolean::*;
pub use buffer::*;
pub use byte::*;
pub use bytes::*;
pub use bytes32::*;
use itertools::Itertools;
use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, Witness, WitnessWrite};
pub use variable::*;

pub use super::uint::uint256::*;
pub use super::uint::uint32::*;
use crate::frontend::builder::CircuitBuilder;

pub trait CircuitVariable: Debug + Clone + Sized + Sync + Send + 'static {
    /// The underlying type of the variable if it were not in a circuit.
    type ValueType<F: RichField>: Debug;

    /// Initializes the variable with no value in the circuit.
    fn init<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self;

    /// Initializes the variable with a constant value in the circuit.
    fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        value: Self::ValueType<F>,
    ) -> Self;

    /// Serializes the circuit variable to targets.
    fn targets(&self) -> Vec<Target> {
        self.variables().into_iter().map(|v| v.0).collect()
    }

    /// Deserializes a variable from a list of targets.
    fn from_targets(targets: &[Target]) -> Self {
        Self::from_variables(&targets.iter().map(|t| Variable(*t)).collect_vec())
    }

    /// Serializes the circuit variable to variables.
    fn variables(&self) -> Vec<Variable>;

    /// Deserializes the circuit variable from variables.
    fn from_variables(variables: &[Variable]) -> Self;

    /// Gets the value of the variable from the witness.
    fn get<F: RichField, W: Witness<F>>(&self, witness: &W) -> Self::ValueType<F>;

    /// Sets the value of the variable in the witness.
    fn set<F: RichField, W: WitnessWrite<F>>(&self, witness: &mut W, value: Self::ValueType<F>);

    /// The number of field elements it takes to represent this variable.
    fn nb_elements() -> usize {
        type F = GoldilocksField;
        const D: usize = 2;
        let mut builder = CircuitBuilder::<F, D>::new();
        let variable = builder.init::<Self>();
        variable.variables().len()
    }

    /// Serializes the value to a list of field elements.
    fn elements<F: RichField + Extendable<D>, const D: usize>(value: Self::ValueType<F>) -> Vec<F> {
        let mut builder = CircuitBuilder::<F, D>::new();
        let variable = builder.constant::<Self>(value);
        let variables = variable.variables();
        variables
            .into_iter()
            .map(|v| *builder.constants.get(&v).unwrap())
            .collect()
    }

    /// Deserializes the value to a list of field elements.
    fn from_elements<F: RichField + Extendable<D>, const D: usize>(
        elements: &[F],
    ) -> Self::ValueType<F> {
        let mut builder = CircuitBuilder::<F, D>::new();
        let variable = builder.init::<Self>();
        let variables = variable.variables();
        assert_eq!(variables.len(), elements.len());
        let mut pw = PartialWitness::new();
        for i in 0..elements.len() {
            variables[i].set(&mut pw, elements[i])
        }
        variable.get(&pw)
    }
}

pub trait EvmVariable: CircuitVariable {
    /// The number of bytes it takes to represent this variable.
    fn nb_bytes<F: RichField + Extendable<D>, const D: usize>() -> usize {
        let mut builder = CircuitBuilder::<F, D>::new();
        let variable = builder.init::<Self>();
        variable.encode(&mut builder).len()
    }

    /// The number of bits it takes to represent this variable.
    fn nb_bits<F: RichField + Extendable<D>, const D: usize>() -> usize {
        Self::nb_bytes::<F, D>() * 8
    }

    /// Serializes the variable to a vector of byte variables with len `nb_bytes()`. This
    /// implementation should match the implementation of `abi.encodePacked(...)`.
    fn encode<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<ByteVariable>;

    /// Deserializes the variable to a vector of byte variables with len `nb_bytes()`. This
    /// implementation should match the implementation of `abi.decodePacked(...)`.
    fn decode<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        bytes: &[ByteVariable],
    ) -> Self;

    /// Serializes a value to bytes. This implementation should match the implementation of
    /// `abi.encodePacked(...)`.
    fn encode_value<F: RichField>(value: Self::ValueType<F>) -> Vec<u8>;

    /// Deserializes a value from bytes. This implementation should match the implementation of
    /// `abi.decodePacked(...)`.
    fn decode_value<F: RichField>(bytes: &[u8]) -> Self::ValueType<F>;

    /// Serializes the variable to little endian bits.
    fn to_le_bits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<BoolVariable> {
        let bytes = self.encode(builder);
        let mut bytes = bytes.into_iter().flat_map(|b| b.as_be_bits()).collect_vec();
        bytes.reverse();
        bytes
    }

    /// Serializes the variable to big endian bits.
    fn to_be_bits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<BoolVariable> {
        let mut bits = self.to_le_bits(builder);
        bits.reverse();
        bits
    }
}

pub trait SSZVariable: CircuitVariable {
    fn hash_tree_root<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Bytes32Variable;
}
