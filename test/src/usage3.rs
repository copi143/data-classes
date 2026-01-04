use data_classes::{deps::*, derive::*};

#[derive(ToPrev, ToNext, ToRandom, PartialEq, Eq, Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[data(to-*)]
enum Direction1 {
    North,
    East,
    South,
    West,
}

#[cfg(test)]
mod tests {
    use super::Direction;
    use data_classes::deps::rand;
    use data_classes::{ToNext as _, ToPrev as _, ToRandom as _};

    #[test]
    fn test_to_prev() {
        assert_eq!(Direction::North.get_prev(), Direction::West);
        assert_eq!(Direction::East.get_prev(), Direction::North);
        assert_eq!(Direction::South.get_prev(), Direction::East);
        assert_eq!(Direction::West.get_prev(), Direction::South);
    }

    #[test]
    fn test_to_next() {
        assert_eq!(Direction::North.get_next(), Direction::East);
        assert_eq!(Direction::East.get_next(), Direction::South);
        assert_eq!(Direction::South.get_next(), Direction::West);
        assert_eq!(Direction::West.get_next(), Direction::North);
    }

    #[test]
    fn test_to_random() {
        let mut rng = rand::rng();
        let _ = Direction::random(&mut rng);
        let _ = Direction::North.get_random(&mut rng);
    }
}
