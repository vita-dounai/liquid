use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}
