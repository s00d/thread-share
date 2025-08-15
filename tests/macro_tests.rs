use thread_share::{share, simple_share, SimpleShare, ThreadShare};

#[test]
fn test_share_macro_basic_types() {
    // Test with integer
    let int_share = share!(42);
    assert_eq!(int_share.get(), 42);

    // Test with string
    let string_share = share!("hello");
    assert_eq!(string_share.get(), "hello");

    // Test with boolean
    let bool_share = share!(true);
    assert_eq!(bool_share.get(), true);

    // Test with float
    let float_share = share!(3.14);
    assert_eq!(float_share.get(), 3.14);
}

#[test]
fn test_share_macro_complex_types() {
    // Test with vector
    let vec_share = share!(vec![1, 2, 3, 4, 5]);
    assert_eq!(vec_share.get(), vec![1, 2, 3, 4, 5]);

    // Test with tuple
    let tuple_share = share!((1, "hello", true));
    assert_eq!(tuple_share.get(), (1, "hello", true));

    // Test with array
    let array_share = share!([1, 2, 3]);
    assert_eq!(array_share.get(), [1, 2, 3]);

    // Test with string
    let string_share = share!(String::from("hello world"));
    assert_eq!(string_share.get(), "hello world");
}

#[test]
fn test_share_macro_custom_struct() {
    #[derive(Clone, Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        text: String,
        flag: bool,
    }

    let struct_share = share!(TestStruct {
        value: 42,
        text: "test".to_string(),
        flag: true,
    });

    let result = struct_share.get();
    assert_eq!(result.value, 42);
    assert_eq!(result.text, "test");
    assert_eq!(result.flag, true);
}

#[test]
fn test_share_macro_nested_structs() {
    #[derive(Clone, Debug, PartialEq)]
    struct InnerStruct {
        value: i32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct OuterStruct {
        inner: InnerStruct,
        name: String,
    }

    let nested_share = share!(OuterStruct {
        inner: InnerStruct { value: 100 },
        name: "nested".to_string(),
    });

    let result = nested_share.get();
    assert_eq!(result.inner.value, 100);
    assert_eq!(result.name, "nested");
}

#[test]
fn test_share_macro_enum() {
    #[derive(Clone, Debug, PartialEq)]
    enum TestEnum {
        Unit,
        Tuple(i32, String),
        Struct { value: i32, name: String },
    }

    // Test unit variant
    let unit_share = share!(TestEnum::Unit);
    assert_eq!(unit_share.get(), TestEnum::Unit);

    // Test tuple variant
    let tuple_share = share!(TestEnum::Tuple(42, "hello".to_string()));
    match tuple_share.get() {
        TestEnum::Tuple(v, s) => {
            assert_eq!(v, 42);
            assert_eq!(s, "hello");
        }
        _ => panic!("Expected Tuple variant"),
    }

    // Test struct variant
    let struct_share = share!(TestEnum::Struct {
        value: 100,
        name: "world".to_string(),
    });
    match struct_share.get() {
        TestEnum::Struct { value, name } => {
            assert_eq!(value, 100);
            assert_eq!(name, "world");
        }
        _ => panic!("Expected Struct variant"),
    }
}

#[test]
fn test_share_macro_option() {
    // Test Some variant
    let some_share = share!(Some(42));
    assert_eq!(some_share.get(), Some(42));

    // Test None variant
    let none_share = share!(Option::<i32>::None);
    assert_eq!(none_share.get(), None);
}

#[test]
fn test_share_macro_result() {
    // Test Ok variant
    let ok_share = share!(Ok::<i32, String>(42));
    assert_eq!(ok_share.get(), Ok(42));

    // Test Err variant
    let err_share = share!(Err::<i32, String>("error".to_string()));
    assert_eq!(err_share.get(), Err("error".to_string()));
}

#[test]
fn test_share_macro_collections() {
    // Test HashMap
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert("key1".to_string(), 1);
    map.insert("key2".to_string(), 2);

    let map_share = share!(map);
    let result = map_share.get();
    assert_eq!(result.get("key1"), Some(&1));
    assert_eq!(result.get("key2"), Some(&2));

    // Test HashSet
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);

    let set_share = share!(set);
    let result = set_share.get();
    assert!(result.contains(&1));
    assert!(result.contains(&2));
    assert!(result.contains(&3));
}

#[test]
fn test_share_macro_references() {
    let value = 42;
    let ref_share = share!(&value);
    assert_eq!(*ref_share.get(), 42);
}

#[test]
fn test_share_macro_function_calls() {
    fn create_value() -> i32 {
        42
    }
    fn create_string() -> String {
        "hello".to_string()
    }

    let value_share = share!(create_value());
    assert_eq!(value_share.get(), 42);

    let string_share = share!(create_string());
    assert_eq!(string_share.get(), "hello");
}

#[test]
fn test_share_macro_arithmetic() {
    let calc_share = share!(2 + 2 * 3);
    assert_eq!(calc_share.get(), 8);

    let complex_calc = share!((10 - 5) * 2 + 1);
    assert_eq!(complex_calc.get(), 11);
}

#[test]
fn test_simple_share_macro_basic_types() {
    // Test with integer
    let int_share = simple_share!(42);
    assert_eq!(int_share.get(), 42);

    // Test with string
    let string_share = simple_share!("hello");
    assert_eq!(string_share.get(), "hello");

    // Test with boolean
    let bool_share = simple_share!(true);
    assert_eq!(bool_share.get(), true);

    // Test with float
    let float_share = simple_share!(3.14);
    assert_eq!(float_share.get(), 3.14);
}

#[test]
fn test_simple_share_macro_complex_types() {
    // Test with vector
    let vec_share = simple_share!(vec![1, 2, 3, 4, 5]);
    assert_eq!(vec_share.get(), vec![1, 2, 3, 4, 5]);

    // Test with tuple
    let tuple_share = simple_share!((1, "hello", true));
    assert_eq!(tuple_share.get(), (1, "hello", true));

    // Test with array
    let array_share = simple_share!([1, 2, 3]);
    assert_eq!(array_share.get(), [1, 2, 3]);

    // Test with string
    let string_share = simple_share!(String::from("hello world"));
    assert_eq!(string_share.get(), "hello world");
}

#[test]
fn test_simple_share_macro_custom_struct() {
    #[derive(Clone, Debug, PartialEq)]
    struct TestStruct {
        value: i32,
        text: String,
        flag: bool,
    }

    let struct_share = simple_share!(TestStruct {
        value: 42,
        text: "test".to_string(),
        flag: true,
    });

    let result = struct_share.get();
    assert_eq!(result.value, 42);
    assert_eq!(result.text, "test");
    assert_eq!(result.flag, true);
}

#[test]
fn test_macro_type_inference() {
    // Test that macros properly infer types
    let int_share = share!(42);
    let _: ThreadShare<i32> = int_share;

    let string_share = share!("hello");
    let _: ThreadShare<&str> = string_share;

    let simple_int_share = simple_share!(42);
    let _: SimpleShare<i32> = simple_int_share;

    let simple_string_share = simple_share!("hello");
    let _: SimpleShare<&str> = simple_string_share;
}

#[test]
fn test_macro_operations() {
    let share = share!(vec![1, 2, 3]);

    // Test that we can perform operations on the shared data
    share.update(|v| v.push(4));
    assert_eq!(share.get(), vec![1, 2, 3, 4]);

    let simple = simple_share!(42);
    simple.set(100);
    assert_eq!(simple.get(), 100);
}

#[test]
fn test_macro_edge_cases() {
    // Test with unit type
    let unit_share = share!(());
    assert_eq!(unit_share.get(), ());

    // Test with empty vector
    let empty_vec_share = share!(Vec::<i32>::new());
    assert_eq!(empty_vec_share.get(), vec![] as Vec<i32>);

    // Test with empty string
    let empty_string_share = share!(String::new());
    assert_eq!(empty_string_share.get(), "");

    // Test with zero
    let zero_share = share!(0);
    assert_eq!(zero_share.get(), 0);
}

#[test]
fn test_macro_nested_macros() {
    // Test that macros can be nested
    let inner_share = share!(42);
    let outer_share = share!(inner_share);

    let result = outer_share.get();
    assert_eq!(result.get(), 42);
}

#[test]
fn test_macro_with_generics() {
    // Test with generic types
    fn create_share<T: Clone>(value: T) -> ThreadShare<T> {
        share!(value)
    }

    let int_share = create_share(42);
    assert_eq!(int_share.get(), 42);

    let string_share = create_share("hello");
    assert_eq!(string_share.get(), "hello");
}

#[test]
fn test_macro_consistency() {
    // Test that both macros produce consistent results
    let share_result = share!(42);
    let simple_result = simple_share!(42);

    assert_eq!(share_result.get(), simple_result.get());

    // Test that both can be updated
    share_result.set(100);
    simple_result.set(100);

    assert_eq!(share_result.get(), 100);
    assert_eq!(simple_result.get(), 100);
}
