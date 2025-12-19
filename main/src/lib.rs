pub use data_classes_derive::*;

/// Trait for enums that can get the previous variant in a circular manner.
pub trait ToPrev: Sized {
    fn get_prev(&self) -> Self;

    fn switch_to_prev(&mut self) {
        *self = self.get_prev();
    }
}

/// Trait for enums that can get the next variant in a circular manner.
pub trait ToNext: Sized {
    fn get_next(&self) -> Self;

    fn switch_to_next(&mut self) {
        *self = self.get_next();
    }
}

/// Trait for enums that can get a random variant.
#[cfg(feature = "rand")]
pub trait ToRandom: Sized {
    fn random<R: rand::Rng + ?Sized>(rng: &mut R) -> Self;

    fn get_random<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self {
        Self::random(rng)
    }

    fn switch_to_random<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        *self = Self::random(rng);
    }
}
