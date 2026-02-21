//! Fundamental type system for Dugong CFD solver
//!
//! Provides dimension-aware quantities, tensor types, and field value traits.

pub mod dimension;
pub mod tensor;
pub mod traits;

pub use dimension::{
    Density, Dim, DynamicViscosity, KinematicViscosity, Length, Mass, Pressure, Quantity, Time,
    Velocity,
};
pub use traits::{FieldValue, HasDiv, HasGrad};
