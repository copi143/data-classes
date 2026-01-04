pub mod derive {
    pub use data_classes_derive::*;
}

pub mod deps {
    #[cfg(feature = "rand")]
    pub use rand;

    #[cfg(feature = "serde")]
    pub extern crate serde;

    #[cfg(feature = "rkyv")]
    pub use rkyv;

    #[cfg(feature = "bytemuck")]
    pub use bytemuck;
}

/// Trait for enums that can get the previous variant in a circular manner.
pub trait ToPrev: Sized {
    /// Gets the previous variant.
    fn get_prev(&self) -> Self;

    /// Switches to the previous variant.
    fn switch_to_prev(&mut self) {
        *self = self.get_prev();
    }
}

/// Trait for enums that can get the next variant in a circular manner.
pub trait ToNext: Sized {
    /// Gets the next variant.
    fn get_next(&self) -> Self;

    /// Switches to the next variant.
    fn switch_to_next(&mut self) {
        *self = self.get_next();
    }
}

/// Trait for enums that can get a random variant.
#[cfg(feature = "rand")]
pub trait ToRandom: Sized {
    /// Gets a random variant.
    fn random<R: rand::Rng + ?Sized>(rng: &mut R) -> Self;

    /// Gets a random variant.
    fn get_random<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self {
        Self::random(rng)
    }

    /// Switches to a random variant.
    fn switch_to_random<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        *self = Self::random(rng);
    }
}
