use crate::validations::{*};

pub trait Checker<T> {
    fn new(data:T) -> Self;
    fn check(&self) -> Option<error::ValidationError>;
    fn dispatch_struct(&self) -> Vec<field::Field>;
}