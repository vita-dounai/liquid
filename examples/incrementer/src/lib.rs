#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod incrementer {
    use liquid_core::storage;

    #[liquid(storage)]
    struct Incrementer {
        value: storage::Value<u128>,
    }

    #[liquid(methods)]
    impl Incrementer {
        pub fn new(&mut self, init: u128) {
            self.value.initialize(init);
        }

        pub fn inc_by(&mut self, delta: u128) {
            self.value += delta;
        }

        pub fn get(&self) -> u128 {
            *self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn init_works() {
            let contract = Incrementer::new(0);
            assert_eq!(contract.get(), 0);
        }

        #[test]
        fn inc_by_works() {
            let mut contract = Incrementer::new(0);
            contract.inc_by(42);
            assert_eq!(contract.get(), 42);
            contract.inc_by(42);
            assert_eq!(contract.get(), 84);
        }
    }
}
