#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_sym)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_slice)]
#![feature(fmt_internals)]
#![feature(start)]
use core::{
    fmt::{Formatter, Write},
    mem::MaybeUninit,
};

use cortex_m_semihosting::hprintln;
use panic_abort as _;
mod boot;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Some naive buffers.
    let mut mybuf1 = MyBuf {
        buf: [0; 40],
        len: 0,
    };
    let mut mybuf2 = MyBuf {
        buf: [0; 40],
        len: 0,
    };

    // Use the core library's fmt. The code breaks.
    mybuf1.write_fmt(format_args!("{}", 256)).unwrap();
    let s = core::str::from_utf8(&mybuf1.buf[0..mybuf1.len]).unwrap();

    // It prints "2;" on my side.
    hprintln!("{}", s);

    // The same thing happens for the straight forward print.
    // It actually prints "2;". Under the hood it goes through
    // the procedure above as in `mybuf1`.
    hprintln!("{}", 256);

    // It is `_ZN4core3fmt3num3imp7fmt_u32#hash` that breaks it.
    // Source code: https://github.com/rust-lang/rust/blob/3ea44938e21f0d/library/core/src/fmt/num.rs#L211-L274
    //
    // The Rust code leading to bad instruction is:
    // let lut_ptr = DEC_DIGITS_LUT.as_ptr();
    //
    // The bad instructions are:
    // 805b322:    f243 7124     movw    r1, #14116    ; 0x3724
    // 805b326:    f2c0 0101     movt    r1, #1
    // 805b32a:    4648          mov     r0, r9
    // 805b32c:    5840          ldr     r0, [r0, r1]
    // 805b32e:    21c8          movs    r1, #200    ; 0xc8
    // 805b330:    f003 fc29     bl      805eb86 <_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6as_ptr17h58e80ef4ecfdc5b4E>
    //
    // The assembly instructions are using static base (R9) relative addressing (into SRAM),
    // but it shouldn't. DEC_DIGITS_LUT is read-only, so it should be PC-relative
    // addressing (into Flash). The ldr instruction above is reading from some
    // wild memory address.


    // If I copy out the core library's implementation into `print_u32`,
    // it magically compiles to correct code. The below example correctly
    // prints out "256".
    let mut formatter = Formatter::new(&mut mybuf2);
    print_u32(256, false, &mut formatter).unwrap();
    let s = core::str::from_utf8(&mybuf2.buf[0..mybuf2.len]).unwrap();
    hprintln!("{}", s);

    // In my copied version, the Rust code
    // let lut_ptr = DEC_DIGITS_LUT.as_ptr();
    // 
    // compiles to the following instructions:
    // 80007d2:    f241 504a     movw    r0, #5450    ; 0x154a
    // 80007d6:    f2c0 0006     movt    r0, #6
    // 80007da:    4478          add     r0, pc
    // 80007dc:    6800          ldr     r0, [r0, #0]
    // 80007de:    21c8          movs    r1, #200    ; 0xc8
    // 80007e0:    f05e f9d1     bl      805eb86 <_ZN4core5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6as_ptr17h58e80ef4ecfdc5b4E>
    //
    // Surprisingly, this time it uses PC relative addressing to read DEC_DIGITS_LUT.
    // The result is thus correct.

    // Tested compiler version:
    // - Nightly 2022-03-30
    // - Nightly 2022-04-08

    loop {}
}

static DEC_DIGITS_LUT: &[u8; 200] = b"0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";

/// The code is copied from
/// https://github.com/rust-lang/rust/blob/3ea44938e21f0d/library/core/src/fmt/num.rs#L211-L274
fn print_u32(
    mut n: u32,
    is_nonnegative: bool,
    f: &mut core::fmt::Formatter<'_>,
) -> core::fmt::Result {
    // 2^128 is about 3*10^38, so 39 gives an extra byte of space
    let mut buf = [MaybeUninit::<u8>::uninit(); 39];
    let mut curr = buf.len() as isize;
    let buf_ptr = MaybeUninit::slice_as_mut_ptr(&mut buf);
    let lut_ptr = DEC_DIGITS_LUT.as_ptr();

    // SAFETY: Since `d1` and `d2` are always less than or equal to `198`, we
    // can copy from `lut_ptr[d1..d1 + 1]` and `lut_ptr[d2..d2 + 1]`. To show
    // that it's OK to copy into `buf_ptr`, notice that at the beginning
    // `curr == buf.len() == 39 > log(n)` since `n < 2^128 < 10^39`, and at
    // each step this is kept the same as `n` is divided. Since `n` is always
    // non-negative, this means that `curr > 0` so `buf_ptr[curr..curr + 1]`
    // is safe to access.
    unsafe {
        // need at least 16 bits for the 4-characters-at-a-time to work.
        assert!(core::mem::size_of::<u32>() >= 2);

        // eagerly decode 4 characters at a time
        while n >= 10000 {
            let rem = (n % 10000) as isize;
            n /= 10000;

            let d1 = (rem / 100) << 1;
            let d2 = (rem % 100) << 1;
            curr -= 4;

            // We are allowed to copy to `buf_ptr[curr..curr + 3]` here since
            // otherwise `curr < 0`. But then `n` was originally at least `10000^10`
            // which is `10^40 > 2^128 > n`.
            core::ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
            core::ptr::copy_nonoverlapping(lut_ptr.offset(d2), buf_ptr.offset(curr + 2), 2);
        }

        // if we reach here numbers are <= 9999, so at most 4 chars long
        let mut n = n as isize; // possibly reduce 64bit math

        // decode 2 more chars, if > 2 chars
        if n >= 100 {
            let d1 = (n % 100) << 1;
            n /= 100;
            curr -= 2;
            core::ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
        }

        // decode last 1 or 2 chars
        if n < 10 {
            curr -= 1;
            *buf_ptr.offset(curr) = (n as u8) + b'0';
        } else {
            let d1 = n << 1;
            curr -= 2;
            core::ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf_ptr.offset(curr), 2);
        }
    }

    // SAFETY: `curr` > 0 (since we made `buf` large enough), and all the chars are valid
    // UTF-8 since `DEC_DIGITS_LUT` is
    let buf_slice = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            buf_ptr.offset(curr),
            buf.len() - curr as usize,
        ))
    };
    f.pad_integral(is_nonnegative, "", buf_slice)
}

struct MyBuf {
    buf: [u8; 40],
    len: usize,
}

impl Write for MyBuf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let buflen = self.buf.len();
        let sslice = s.as_bytes();
        let slicelen = sslice.len();
        if slicelen >= buflen {
            self.buf.copy_from_slice(&sslice[0..buflen]);
            self.len = buflen;
        } else {
            let partialbuf = self.buf.split_at_mut(slicelen).0;
            partialbuf.copy_from_slice(sslice);
            self.len = slicelen;
        }
        Ok(())
    }
}
