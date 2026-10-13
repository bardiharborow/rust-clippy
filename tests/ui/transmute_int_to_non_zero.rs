#![feature(decl_macro)]
#![warn(clippy::transmute_int_to_non_zero)]

use core::num::NonZero;

fn main() {
    let int_u8: u8 = 1;
    let int_u16: u16 = 1;
    let int_u32: u32 = 1;
    let int_u64: u64 = 1;
    let int_u128: u128 = 1;

    let int_i8: i8 = 1;
    let int_i16: i16 = 1;
    let int_i32: i32 = 1;
    let int_i64: i64 = 1;
    let int_i128: i128 = 1;

    let _: NonZero<u8> = unsafe { std::mem::transmute(int_u8) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<u16> = unsafe { std::mem::transmute(int_u16) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<u32> = unsafe { std::mem::transmute(int_u32) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<u64> = unsafe { std::mem::transmute(int_u64) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<u128> = unsafe { std::mem::transmute(int_u128) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<i8> = unsafe { std::mem::transmute(int_i8) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<i16> = unsafe { std::mem::transmute(int_i16) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<i32> = unsafe { std::mem::transmute(int_i32) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<i64> = unsafe { std::mem::transmute(int_i64) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<i128> = unsafe { std::mem::transmute(int_i128) };
    //~^ transmute_int_to_non_zero

    let _: NonZero<u8> = unsafe { NonZero::new_unchecked(int_u8) };
    let _: NonZero<u16> = unsafe { NonZero::new_unchecked(int_u16) };
    let _: NonZero<u32> = unsafe { NonZero::new_unchecked(int_u32) };
    let _: NonZero<u64> = unsafe { NonZero::new_unchecked(int_u64) };
    let _: NonZero<u128> = unsafe { NonZero::new_unchecked(int_u128) };

    let _: NonZero<i8> = unsafe { NonZero::new_unchecked(int_i8) };
    let _: NonZero<i16> = unsafe { NonZero::new_unchecked(int_i16) };
    let _: NonZero<i32> = unsafe { NonZero::new_unchecked(int_i32) };
    let _: NonZero<i64> = unsafe { NonZero::new_unchecked(int_i64) };
    let _: NonZero<i128> = unsafe { NonZero::new_unchecked(int_i128) };

    // Only the explicit types given to `transmute` make this a `u32`, so the suggestion must keep them
    let _ = unsafe { std::mem::transmute::<u32, NonZero<u32>>(5) };
    //~^ transmute_int_to_non_zero
    #[allow(clippy::missing_transmute_annotations)]
    let _ = unsafe { std::mem::transmute::<_, NonZero<u32>>(5u32) };
    //~^ transmute_int_to_non_zero
}

mod alias {
    // `NonZero` itself is not in scope here, so the suggestion must use a full path
    use std::num::NonZeroU32;

    fn f(x: u32) -> NonZeroU32 {
        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }
}

mod glob {
    use std::num::*;

    fn f(x: u32) -> NonZero<u32> {
        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }
}

mod shadowed_in_block {
    use std::num::NonZero;

    fn local_item(x: u32) -> NonZero<u32> {
        // This `NonZero` shadows the import, so the suggestion must use a full path
        #[allow(dead_code)]
        struct NonZero;

        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }

    fn local_import(x: u32) -> NonZero<u32> {
        #[allow(unused_imports)]
        use std::{fmt::Debug as NonZero, mem};

        unsafe { mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }

    fn generic_param<NonZero>(x: u32, _: NonZero) -> std::num::NonZero<u32> {
        // The `NonZero` type parameter shadows the import, so the suggestion must use a full path
        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }

    struct Wrapper<T>(T);

    impl<NonZero> Wrapper<NonZero> {
        fn f(x: u32) -> std::num::NonZero<u32> {
            // The `NonZero` type parameter of the `impl` also shadows the import
            unsafe { std::mem::transmute(x) }
            //~^ transmute_int_to_non_zero
        }
    }

    fn unrelated_glob(x: u32) -> NonZero<u32> {
        #[allow(unused_imports)]
        use std::ops::*;

        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }
}

mod shadowed {
    // This `NonZero` is not the one from `std`, so the suggestion must use a full path
    #[allow(dead_code)]
    struct NonZero;

    fn f(x: u32) -> std::num::NonZero<u32> {
        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }
}

#[clippy::msrv = "1.78"]
mod msrv_before_generic_non_zero {
    use std::num::{NonZeroI8, NonZeroU32};

    fn f(x: u32, y: i8, z: usize) {
        let _: NonZeroU32 = unsafe { std::mem::transmute(x) };
        //~^ transmute_int_to_non_zero
        let _: NonZeroI8 = unsafe { std::mem::transmute(y) };
        //~^ transmute_int_to_non_zero
        // `NonZeroUsize` is not in scope, so the suggestion must use a full path
        let _: std::num::NonZeroUsize = unsafe { std::mem::transmute(z) };
        //~^ transmute_int_to_non_zero
    }
}

#[clippy::msrv = "1.79"]
mod msrv_generic_non_zero {
    use std::num::NonZeroU32;

    fn f(x: u32) -> NonZeroU32 {
        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }
}

mod macros {
    use std::num::NonZero;

    macro_rules! five {
        () => {
            5u32
        };
    }

    fn macro_arg() -> NonZero<u32> {
        unsafe { std::mem::transmute(five!()) }
        //~^ transmute_int_to_non_zero
    }
}

mod local_std {
    // This `std` is not the crate, so the suggestion must use an absolute path
    #[allow(dead_code)]
    mod std {}

    fn f(x: u32) -> ::std::num::NonZero<u32> {
        unsafe { ::std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }
}

mod hygiene {
    macro import() {
        #[allow(unused_imports)]
        use std::num::NonZero;
    }
    import!();

    fn f(x: u32) -> std::num::NonZero<u32> {
        // The import from `import!` is not visible here, so the suggestion must use a full path
        unsafe { std::mem::transmute(x) }
        //~^ transmute_int_to_non_zero
    }
}
