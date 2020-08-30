use rune_testing::*;

#[test]
fn test_anonymous_type_precedence() {
    assert_eq! {
        3,
        test! {
            i64 => r#"
            fn main() {
                fn a() { 1 }
                fn b() { return a(); fn a() { 2 } }
                a() + b()
            }
            "#
        }
    };
}
