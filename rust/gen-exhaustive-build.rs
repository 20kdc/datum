/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

// Generates an exhaustive list of cargo build commands to ensure all feature combos build.

static FLAGS: &'static [&'static str] = &[
    "std",
    "alloc",
    "detailed_errors",
    "serde",
    "_experimental",
    "_serde_test_features"
];

fn main() {
    let max: u32 = 1 << FLAGS.len();
    for i in 0..max {
        print!("cargo build --no-default-features");
        let mut first_flag = true;
        for j in 0..FLAGS.len() {
            let flag: u32 = 1 << j;
            if (i & flag) != 0 {
                if !first_flag {
                    print!(",");
                } else {
                    print!(" -F ");
                    first_flag = false;
                }
                print!("{}", FLAGS[j]);
            }
        }
        println!();
    }
}
